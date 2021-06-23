import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import './frontend-button';

type State = Readonly<{
    loading: boolean;
    userId: string;
    blogPostInputText: string;
}>;

const InitialState: State = {
    loading: false,
    userId: '',
    blogPostInputText: ''
};

class FrontendCreateBlogPost extends HTMLElement {
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
                    display: flex;
                }

                .input-container {
                    padding: .25rem;
                }

                .button-container {
                    padding: .25rem;
                }
            </style>

            <div class="main-container">
                <div class="input-container">
                    <input
                        type="text"
                        .value=${state.blogPostInputText}
                        .disabled=${state.loading}
                        @input=${(e: InputEvent) => this.dispatchEvent(new CustomEvent('blog-post-input-text-changed', {
                            detail: {
                                userId: state.userId,
                                blogPostInputText: (e.target as HTMLInputElement).value
                            },
                            bubbles: true,
                            composed: true
                        }))}
                    >
                </div>

                <div class="button-container">
                    <frontend-button
                        .buttonText=${'Create blog post'}
                        .loading=${state.loading}
                        @button-clicked=${() => this.dispatchEvent(new CustomEvent('create-blog-post', {
                            detail: {
                                userId: this.store.userId
                            },
                            bubbles: true,
                            composed: true
                        }))}
                    ></frontend-button>
                </div>                    
            </div>
        `;
    }
}

window.customElements.define('frontend-create-blog-post', FrontendCreateBlogPost);