import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';

type State = Readonly<{
    buttonText: string;
    ellipsis: string;
    loading: boolean;
    intervalId: number;
}>;

const InitialState: State = {
    buttonText: '',
    ellipsis: '...',
    loading: false,
    intervalId: -1
};

class FrontendButton extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    set loading(loading: boolean) {
        this.store.loading = loading;

        if (
            this.store.loading === true &&
            this.store.intervalId === -1
        ) {
            const intervalId = setInterval(() => {
                this.setEllipsis();
            }, 500);

            this.store.intervalId = intervalId;
        }

        if (this.store.loading === false) {
            clearInterval(this.store.intervalId);
            this.store.intervalId = InitialState.intervalId;
        }
    }

    setEllipsis() {
        if (this.store.ellipsis === '') {
            this.store.ellipsis = '.';
            return;
        }

        if (this.store.ellipsis === '.') {
            this.store.ellipsis = '..';
            return;
        }

        if (this.store.ellipsis === '..') {
            this.store.ellipsis = '...';
            return;
        }

        if (this.store.ellipsis === '...') {
            this.store.ellipsis = '';
            return;
        }
    }

    render(state: State) {
        const buttonText = state.loading === true ? `Saving${state.ellipsis}` : state.buttonText;

        return html`
            <style>
                .main-container {
                    width: 100%;
                    height: 100%;
                }
            </style>

            <div class="main-container">
                <button
                    @click=${() => this.dispatchEvent(new CustomEvent('button-clicked', {
                        bubbles: true,
                        composed: true
                    }))}
                    .disabled=${state.loading}
                >
                    ${buttonText}
                </button>
            </div>
        `;
    }
}

window.customElements.define('frontend-button', FrontendButton);