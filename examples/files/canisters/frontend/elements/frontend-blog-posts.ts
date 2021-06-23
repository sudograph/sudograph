import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    BlogPost
} from '../types/index.d';
import './frontend-create-blog-post';

type State = Readonly<{
    userId: string;
    loading: boolean;
    blogPosts: ReadonlyArray<BlogPost>;
    blogPostInputText: string;
}>;

const InitialState: State = {
    userId: '',
    loading: false,
    blogPosts: [],
    blogPostInputText: ''
};

class FrontendBlogPosts extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    render(state: State) {
        return html`
            <style>
                .main-container {
                    width: 100%;
                    height: 100%;
                }

                .frontend-create-blog-post-container {
                    padding-bottom: 1rem;
                }

                .blog-posts-container {
                    padding-left: 1rem;
                }
            </style>

            <div class="main-container">
                <h3>Blog Posts</h3>

                <div
                    ?hidden=${state.userId === 'NOT_SET'}
                    class="frontend-create-blog-post-container"
                >
                    <frontend-create-blog-post
                        .userId=${state.userId}
                        .loading=${state.loading}
                        .blogPostInputText=${state.blogPostInputText}
                    ></frontend-create-blog-post>
                </div>

                <div class="blog-posts-container">
                    ${state.blogPosts.map((blogPost) => {
                        return html`
                            <div>
                                ${blogPost.title}
                            </div>

                            <br>
                        `;
                    })}
                </div>
            </div>
        `;
    }
}

window.customElements.define('frontend-blog-posts', FrontendBlogPosts);