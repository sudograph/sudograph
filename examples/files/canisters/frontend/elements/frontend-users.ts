import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    User,
    BlogPostInputTexts,
    BlogPostsAreSaving
} from '../types/index.d';
import './frontend-blog-posts';

type State = Readonly<{
    blogPostsAreSaving: BlogPostsAreSaving;
    blogPostInputTexts: BlogPostInputTexts;
    users: ReadonlyArray<User>;
}>;

const InitialState: State = {
    blogPostsAreSaving: {},
    blogPostInputTexts: {},
    users: []
};

class FrontendUsers extends HTMLElement {
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
            </style>

            <div class="main-container">
                <h1>Users</h1>

                <div>
                    ${state.users.map((user) => {
                        return html`
                            <h2>${user.username}</h2>

                            <frontend-blog-posts
                                .userId=${user.id}
                                .loading=${state.blogPostsAreSaving[user.id] ?? false}
                                .blogPostInputText=${state.blogPostInputTexts[user.id] ?? ''}
                                .blogPosts=${user.blogPosts}
                            ></frontend-blog-posts>
                        `;
                    })}
                </div>
            </div>
        `;
    }
}

window.customElements.define('frontend-users', FrontendUsers);