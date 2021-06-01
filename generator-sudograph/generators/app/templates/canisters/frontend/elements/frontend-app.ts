import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    User
} from '../types/index.d';
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

type State = Readonly<{
    users: ReadonlyArray<User>;
    usernameInputText: string;
}>;

const InitialState: State = {
    users: [],
    usernameInputText: ''
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

    async createAndUpdateUsers() {
        const success = await createUser(this.store.usernameInputText);
    
        if (success === true) {
            await this.fetchAndSetUsers();
        }
        else {
            alert('User was not saved successfully');
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
                <div>
                    <h1>Add user</h1>
    
                    <div>
                        Username: <input type="text" @input=${(e: InputEvent) => this.store.usernameInputText = (e.target as HTMLInputElement).value}>
                        <button @click=${() => this.createAndUpdateUsers()}>Create user</button>
                    </div>
                </div>

                <div>
                    <h1>Users</h1>
    
                    <div>
                        ${state.users.map((user) => {
                            return html`
                                <div>Username: ${user.username}</div>
                                <div>Blog posts:</div>
    
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