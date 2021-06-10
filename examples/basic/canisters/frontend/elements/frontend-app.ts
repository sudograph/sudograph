import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    BlogPostsAreSaving,
    BlogPostInputTextChangedEvent,
    BlogPostInputTexts,
    CreateBlogPostEvent,
    User,
    UsernameInputTextChangedEvent
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
    blogPostsAreSaving: BlogPostsAreSaving;
    blogPostInputTexts: BlogPostInputTexts;
    userIsSaving: boolean;
    usernameInputText: string;
    users: ReadonlyArray<User>;
}>;

const InitialState: State = {
    blogPostsAreSaving: {},
    blogPostInputTexts: {},
    userIsSaving: false,
    usernameInputText: '',
    users: []
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
        this.store.userIsSaving = true;

        this.addUnsavedUserToUsers();
        
        const success = await createUser(this.store.usernameInputText);
        
        if (success === true) {
            this.store.usernameInputText = '';
            await this.fetchAndSetUsers();
        }
        else {
            this.removeUnsavedUserFromUsers();
            alert('User was not saved successfully');
        }

        this.store.userIsSaving = false;
    }

    async createBlogPostAndUpdateUsers(userId: string) {
        this.setBlogPostIsSaving(userId, true);

        this.addUnsavedBlogPostToUsers(userId);

        const success = await createBlogPost(
            userId,
            this.store.blogPostInputTexts[userId]
        );

        if (success === true) {
            this.setBlogPostInputText(userId, '');
            await this.fetchAndSetUsers();
        }
        else {
            this.removeUnsavedBlogPostFromUsers(userId);
            alert('Blog post was not saved successfully');
        }

        this.setBlogPostIsSaving(userId, false);
    }

    addUnsavedUserToUsers() {
        this.store.users = [
            {
                id: 'NOT_SET',
                username: this.store.usernameInputText,
                blogPosts: []
            },
            ...this.store.users
        ];
    }

    removeUnsavedUserFromUsers() {
        this.store.users = this.store.users.filter((user) => user.id !== 'NOT_SET');
    }

    setBlogPostIsSaving(
        userId: string,
        isSaving: boolean
    ) {
        this.store.blogPostsAreSaving = {
            ...this.store.blogPostsAreSaving,
            [userId]: isSaving
        };
    }

    addUnsavedBlogPostToUsers(userId: string) {
        this.store.users = this.store.users.map((user) => {
            if (user.id === userId) {
                return {
                    ...user,
                    blogPosts: [
                        ...user.blogPosts,
                        {
                            id: 'NOT_SET',
                            title: this.store.blogPostInputTexts[userId],
                            publishedAt: new Date(),
                            author: user
                        }
                    ]
                };
            }

            return user;
        });
    }

    removeUnsavedBlogPostFromUsers(userId: string) {
        this.store.users = this.store.users.map((user) => {
            if (user.id === userId) {
                return {
                    ...user,
                    blogPosts: user.blogPosts.filter((blogPost) => blogPost.id !== 'NOT_SET')
                };
            }

            return user;
        });
    }

    setBlogPostInputText(
        userId: string,
        blogPostInputText: string
    ) {
        this.store.blogPostInputTexts = {
            ...this.store.blogPostInputTexts,
            [userId]: blogPostInputText
        };
    }

    render(state: State) {
        return html`
            <style>
                .main-container {
                    width: 100%;
                    height: 100%;
                    display: flex;
                }

                .frontend-create-user-container {
                    flex: 1;
                }

                .frontend-users-container {
                    flex: 1;
                }
            </style>

            <div class="main-container">
                <div class="frontend-create-user-container">
                    <frontend-create-user
                        .loading=${state.userIsSaving}
                        .usernameInputText=${state.usernameInputText}
                        @create-user=${() => this.createUserAndUpdateUsers()}
                        @username-input-text-changed=${(e: UsernameInputTextChangedEvent) => this.store.usernameInputText = e.detail.usernameInputText}
                    ></frontend-create-user>
                </div>

                <div class="frontend-users-container">
                    <frontend-users
                        .blogPostsAreSaving=${state.blogPostsAreSaving}
                        .blogPostInputTexts=${state.blogPostInputTexts}
                        .users=${state.users}
                        @blog-post-input-text-changed=${(e: BlogPostInputTextChangedEvent) => this.setBlogPostInputText(e.detail.userId, e.detail.blogPostInputText)}
                        @create-blog-post=${(e: CreateBlogPostEvent) => this.createBlogPostAndUpdateUsers(e.detail.userId)}
                    ></frontend-users>
                </div>
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