import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import './frontend-button';

type State = Readonly<{
    loading: boolean;
    usernameInputText: string;
}>;

const InitialState: State = {
    loading: false,
    usernameInputText: ''
};

class FrontendCreateUser extends HTMLElement {
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

                .input-and-button-container {
                    display: flex;
                }

                .label-container {
                    padding: .25rem;
                }

                .input-container {
                    padding: .25rem;
                }

                .button-container {
                    padding: .25rem;
                }
            </style>

            <div class="main-container">
                <h1>Create user</h1>

                <div class="input-and-button-container">
                    <div class="label-container">Username:</div>

                    <div class="input-container">
                        <input
                            type="text"
                            .value=${state.usernameInputText}
                            .disabled=${state.loading}
                            @input=${(e: InputEvent) => this.dispatchEvent(new CustomEvent('username-input-text-changed', {
                                detail: {
                                    usernameInputText: (e.target as HTMLInputElement).value
                                },
                                bubbles: true,
                                composed: true
                            }))}
                        >
                    </div>

                    <div class="button-container">
                        <frontend-button
                            .buttonText=${'Create user'}
                            .loading=${state.loading}
                            @button-clicked=${() => this.dispatchEvent(new CustomEvent('create-user', {
                                bubbles: true,
                                composed: true
                            }))}
                        ></frontend-button>
                    </div>                    
                </div>
            </div>
        `;
    }
}

window.customElements.define('frontend-create-user', FrontendCreateUser);