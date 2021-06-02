import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    BlogPostInputTexts,
    BlogPostInputTextChangedEvent,
    User,
    UsernameInputTextChangedEvent,
    CreateBlogPostEvent
} from '../types/index.d';
import {
    gql,
    sudograph
} from 'sudograph';
import './frontend-create-user';
import './frontend-users';

const {
    query,
    mutation
} = sudograph({
    canisterId: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
});

type State = Readonly<{
    loading: boolean;
    users: ReadonlyArray<User>;
    usernameInputText: string;
    blogPostInputTexts: BlogPostInputTexts;
}>;

const InitialState: State = {
    loading: false,
    users: [],
    usernameInputText: '',
    blogPostInputTexts: {}
};

class FrontendApp extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    async connectedCallback() {
        await this.fetchAndSetUsers();
    }

    async fetchAndSetUsers() {
        const users = await fetchUsers();
        this.store.users = users;
    }

    async createUserAndUpdateUsers() {
        // Here we perform an optimistic update
        // We assume that the update call to the canister will succeed
        // We immediately update the users array 
        this.store.users = [
            {
                id: 'NOT_SET',
                username: this.store.usernameInputText,
                blogPosts: []
            },
            ...this.store.users
        ];
        
        this.store.loading = true;
        
        const success = await createUser(this.store.usernameInputText);
        
        if (success === true) {
            // If the update call succeeds, then we clear the input
            this.store.usernameInputText = '';
            await this.fetchAndSetUsers();
        }
        else {
            // If the update call does not succeed, we remove the optimistically created user
            // The input will still have the username text in it
            // This will allow the user to try again
            this.store.users = this.store.users.filter((user) => user.id !== 'NOT_SET');
            alert('User was not saved successfully');
        }

        this.store.loading = false;
    }

    async createBlogPostAndUpdateUsers(userId: string) {
        // TODO add in loading
        // TODO add in input clearing
        const success = await createBlogPost(
            userId,
            this.store.blogPostInputTexts[userId]
        );

        if (success === true) {
            this.store.blogPostInputTexts = {
                ...this.store.blogPostInputTexts,
                [userId]: '' // TODO or we could remove the key using that destructure remove syntax
            };
            await this.fetchAndSetUsers();
        }
        else {
            alert('Blog post was not saved successfully');
        }
    }

    render(state: State) {
        return html`
            <style>
                .main-container {
                    width: 100%;
                    height: 100%;
                }
            </style>

            <div class="main-container">
                <frontend-create-user
                    .loading=${state.loading}
                    .usernameInputText=${state.usernameInputText}
                    @username-input-text-changed=${(e: UsernameInputTextChangedEvent) => this.store.usernameInputText = e.detail.usernameInputText}
                    @create-user=${() => this.createUserAndUpdateUsers()}
                ></frontend-create-user>

                <frontend-users
                    .users=${state.users}
                    .blogPostInputTexts=${state.blogPostInputTexts}
                    @blog-post-input-text-changed=${(e: BlogPostInputTextChangedEvent) => this.store.blogPostInputTexts = {
                        ...this.store.blogPostInputTexts,
                        [e.detail.userId]: e.detail.blogPostInputText
                    }}
                    @create-blog-post=${(e: CreateBlogPostEvent) => this.createBlogPostAndUpdateUsers(e.detail.userId)}
                ></frontend-users>
            </div>
        `;
    }
}

window.customElements.define('frontend-app', FrontendApp);

async function fetchUsers(): Promise<ReadonlyArray<User>> {
    const result = await query(gql`
        query {
            readUser(input: {}) {
                id
                username
                blogPosts {
                    id
                    title
                }
            }
        }
    `);

    console.log('fetchUsers result', result);

    return result.data.readUser; 
}

async function createUser(username: string): Promise<boolean> {
    const result = await mutation(gql`
        mutation ($username: String!) {
            createUser(input: {
                username: $username
            }) {
                id
            }
        }
    `, {
        username
    });

    console.log('createUser result', result);

    const userId = result.data?.createUser?.[0]?.id;

    if (
        userId !== null &&
        userId !== undefined
    ) {
        return true;
    }
    else {
        return false;
    }
}

async function createBlogPost(
    userId: string,
    title: string
): Promise<boolean> {
    const result = await mutation(gql`
        mutation (
            $userId: String!
            $title: String!
            $publishedAt: Date!
        ) {
            createBlogPost(input: {
                author: {
                    connect: $userId
                }
                title: $title
                publishedAt: $publishedAt
            }) {
                id
            }
        }
    `, {
        userId,
        title,
        publishedAt: new Date().toISOString()
    });

    console.log('createBlogPost result', result);

    const blogPostId = result.data?.createBlogPost?.[0]?.id;

    if (
        blogPostId !== null &&
        blogPostId !== undefined
    ) {
        return true;
    }
    else {
        return false;
    }
}