import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    User,
    BlogPostInputTexts
} from '../types/index.d';
import './frontend-create-blog-post';

type State = Readonly<{
    users: ReadonlyArray<User>;
    blogPostInputTexts: BlogPostInputTexts;
}>;

const InitialState: State = {
    users: [],
    blogPostInputTexts: {}
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
                            <div>Username: ${user.username}</div>
                            <div>Blog posts:</div>

                            <div>
                                <frontend-create-blog-post
                                    .userId=${user.id}
                                    .blogPostInputText=${state.blogPostInputTexts[user.id] ?? ''}
                                ></frontend-create-blog-post>
                            </div>

                            <div>
                                ${user.blogPosts.map((blogPost) => {
                                    return html`<div>${blogPost.title}</div>`;
                                })}
                            </div>

                            <br>
                        `;
                    })}
                </div>
            </div>
        `;
    }
}

window.customElements.define('frontend-users', FrontendUsers);