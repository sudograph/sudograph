import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import { BlogPost } from '../types';

type State = {
    blogPost: BlogPost | 'NOT_SET';
};

const InitialState: State = {
    blogPost: 'NOT_SET'
};

class SudoBlogPost extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    render(state: State) {
        if (state.blogPost === 'NOT_SET') {
            return html`Loading...`;
        }

        return html`
            <style>
                .main-container {
                    width: 100%;
                    height: 100%;
                }
            </style>

            <div class="main-container">
                <h1>${state.blogPost.title}</h1>
                <h2>Published: ${state.blogPost.published_at}</h2>
                <div>${state.blogPost.body}</div>
            </div>
        `;
    }
}

window.customElements.define('sudo-blog-post', SudoBlogPost);