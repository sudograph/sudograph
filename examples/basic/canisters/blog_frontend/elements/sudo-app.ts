import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import { BlogPost } from '../types';
import graphql from 'ic:canisters/graphql';
import './sudo-blog-post.ts';
import './sudo-draft.ts';

type State = Readonly<{
    blogPosts: ReadonlyArray<BlogPost>;
    drafts: ReadonlyArray<BlogPost>;
}>;

const InitialState: State = {
    blogPosts: [],
    drafts: []
};

class SudoApp extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    async connectedCallback() {
        await this.fetchAndSetBlogPostsAndDrafts();
    }

    async fetchAndSetBlogPostsAndDrafts() {
        let promises: ReadonlyArray<Promise<ReadonlyArray<BlogPost>>> = [
            fetchBlogPosts(),
            fetchDrafts()
        ];

        const results = await Promise.all(promises);

        const blogPosts: ReadonlyArray<BlogPost> = results[0];
        const drafts: ReadonlyArray<BlogPost> = results[1];

        this.store.blogPosts = blogPosts;
        this.store.drafts = drafts;
    }
    
    createNewDraft() {
        this.store.drafts = [
            {
                id: Symbol('NOT_SET'),
                body: '',
                created_at: new Date().toISOString(),
                live: false,
                num_words: 0,
                published_at: null,
                title: '',
                updated_at: new Date().toISOString()
            },
            ...this.store.drafts
        ];
    }

    async draftPublished(e) {
        const publishedDraftId: string | symbol = e.detail;

        await this.fetchAndSetBlogPostsAndDrafts();

        this.store.drafts = this.store.drafts.filter((draft) => {
            return draft.id !== publishedDraftId;
        });
    }

    render(state: State) {
        return html`
            <style>
                html {
                    margin: 0;
                    font-family: sans-serif;
                }

                body {
                    margin: 0;
                }

                .main-container {
                    width: 100%;
                    height: 100%;
                    display: flex;
                }

                .blog-posts-container {
                    display: flex;
                    flex-direction: column;
                    flex: 1;
                }

                .drafts-container {
                    display: flex;
                    flex-direction: column;
                    flex: 1;
                }

                .blog-post-container {
                    box-shadow: 0px 0px 5px grey;
                }
            </style>

            <div class="main-container">
                <div class="blog-posts-container">
                    <div style="display: flex; justify-content: center">
                        <h1>Blog Posts</h1>
                    </div>
                    
                    <div>
                        ${state.blogPosts.map((blogPost) => {
                            return html`
                                <div class="blog-post-container">
                                    <sudo-blog-post .blogPost=${blogPost}></sudo-blog-post>
                                </div>
                            `;
                        })}
                    </div>
                </div>

                <div class="drafts-container">
                    <div style="display: flex; justify-content: center">
                        <h1>Drafts</h1>
                        <button @click=${() => this.createNewDraft()}>New Draft</button>
                    </div>

                    <div>
                        ${state.drafts.map((draft) => {
                            return html`
                                <div class="blog-post-container">
                                    <sudo-draft
                                        .draft=${draft}
                                        @draft-published=${async (e) => await this.draftPublished(e)}
                                    ></sudo-draft>
                                </div>
                            `;
                        })}
                    </div>
                </div>
            </div>

        `;
    }
}

window.customElements.define('sudo-app', SudoApp);

async function fetchBlogPosts(): Promise<ReadonlyArray<BlogPost>> {
    const resultString = await graphql.graphql_query(`
        query {
            readBlogPost(input: {
                live: {
                    eq: true
                }
            }) {
                id
                body
                created_at
                live
                num_words
                published_at
                title
                updated_at
            }
        }
    `);

    const resultJSON = JSON.parse(resultString);

    return resultJSON.data.readBlogPost;
}

async function fetchDrafts(): Promise<ReadonlyArray<BlogPost>> {
    const resultString = await graphql.graphql_query(`
        query {
            readBlogPost(input: {
                live: {
                    eq: false
                }
            }) {
                id
                body
                created_at
                live
                num_words
                published_at
                title
                updated_at
            }
        }
    `);

    const resultJSON = JSON.parse(resultString);

    return resultJSON.data.readBlogPost;
}