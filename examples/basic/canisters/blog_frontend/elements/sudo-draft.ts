import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import { BlogPost } from '../types';
import {
    gql,
    sudograph
} from 'sudograph';

const {
    query,
    mutation
} = sudograph({
    canisterId: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
});

type State = {
    bodyValue: string;
    draft: BlogPost | 'NOT_SET';
    titleValue: string;
};

const InitialState: State = {
    bodyValue: '',
    draft: 'NOT_SET',
    titleValue: ''
};

class SudoDraft extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    async saveDraft() {
        if (this.store.draft === 'NOT_SET') {
            return;
        }

        const newDraft: BlogPost = {
            ...this.store.draft,
            body: this.store.bodyValue,
            live: this.store.draft.live,
            title: this.store.titleValue
        };

        const draftId: string = await updateDraft(newDraft);

        this.store.draft = {
            ...newDraft,
            id: draftId
        };
    }

    // TODO publishing an already saved draft does not seem to work correctly
    // TODO figure out how to iterate more quickly on the frontend...we might want to not use
    // TODO the replica in development if we want to move quickly
    async publishDraft() {
        if (this.store.draft === 'NOT_SET') {
            return;
        }

        const newDraft: BlogPost = {
            ...this.store.draft,
            body: this.store.bodyValue,
            live: true,
            title: this.store.titleValue
        };

        const draftId: string = await updateDraft(newDraft);

        this.store.draft = {
            ...newDraft,
            id: draftId
        };

        // TODO instantly update instead of doing another network request
        this.dispatchEvent(new CustomEvent('draft-published', {
            detail: this.store.draft.id
        }));
    }

    render(state: State) {
        if (state.draft === 'NOT_SET') {
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
                <input
                    id="title-input"
                    type="text"
                    .value=${state.draft.title}
                    @input=${(e: InputEvent) => this.store.titleValue = (e.target as HTMLInputElement).value}
                >
                <div>Last updated: ${state.draft.updated_at}</div>
                <textarea
                    id="body-textarea"
                    .value=${state.draft.body}
                    @input=${(e: InputEvent) => this.store.bodyValue = (e.target as HTMLTextAreaElement).value}
                ></textarea>

                <button @click=${() => this.saveDraft()}>Save</button>
                <button @click=${() => this.publishDraft()}>Publish</button>
            </div>
        `;
    }
}

window.customElements.define('sudo-draft', SudoDraft);

async function updateDraft(draft: BlogPost): Promise<string> {
    if (typeof draft.id === 'symbol') {
        const result = await mutation(gql`
            mutation (
                $body: String!
                $created_at: Date!
                $live: Boolean!
                $num_words: Int!
                $title: String!
                $updated_at: Date!
            ) {
                createBlogPost(input: {
                    body: $body
                    created_at: $created_at
                    live: $live
                    num_words: $num_words
                    published_at: null
                    title: $title
                    updated_at: $updated_at
                }) {
                    id
                }
            }
        `, {
            body: draft.body,
            created_at: new Date().toISOString(),
            live: draft.live,
            num_words: draft.body.split(' ').length,
            title: draft.title,
            updated_at: new Date().toISOString()
        });

        console.log('result', result);

        return result.data.createBlogPost.id;
    }
    else {
        const result = await mutation(gql`
            mutation (
                $id: ID!
                $body: String!
                $live: Boolean!
                $num_words: Int!
                $published_at: Date!
                $title: String!
                $updated_at: Date!
            ) {
                updateBlogPost(input: {
                    id: $id
                    body: $body
                    live: $live
                    num_words: $num_words
                    published_at: $published_at
                    title: $title
                    updated_at: $updated_at
                }) {
                    id
                }
            }
        `, {
            id: draft.id,
            body: draft.body,
            live: draft.live,
            num_words: draft.body.split(' ').length,
            published_at: new Date().toISOString(),
            title: draft.title,
            updated_at: new Date().toISOString()
        });

        console.log('result', result);

        return result.data.updateBlogPost.id;
    }
}