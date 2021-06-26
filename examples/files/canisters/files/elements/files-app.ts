import {
    html,
    render as litRender
} from 'lit-html';
import { createObjectStore } from 'reduxular';
import {
    gql,
    sudograph
} from 'sudograph';
import {
    FileMeta,
    ChunkInfo,
    UploadState,
    DownloadState
} from '../types/index.d';
import { GRAPHQL_CANISTER_ID } from '../utilities/environment';

const {
    query,
    mutation
} = sudograph({
    canisterId: GRAPHQL_CANISTER_ID
});

type State = Readonly<{
    fileMetas: ReadonlyArray<FileMeta>;
    creatingFileMeta: boolean;
    uploadStates: {
        [fileId: string]: UploadState;
    };
    downloadStates: {
        [fileId: string]: DownloadState;
    };
}>;

const InitialState: State = {
    fileMetas: [],
    creatingFileMeta: false,
    uploadStates: {},
    downloadStates: {}
};

class FilesApp extends HTMLElement {
    shadow = this.attachShadow({
        mode: 'closed'
    });
    store = createObjectStore(InitialState, (state: State) => litRender(this.render(state), this.shadow), this);

    async connectedCallback() {
        await this.fetchAndSetFileMetas();
    }
    
    async fetchAndSetFileMetas() {
        this.store.fileMetas = await fetchFileMetas();

        this.store.fileMetas.forEach((fileMeta) => {
            this.store.uploadStates = {
                ...this.store.uploadStates,
                [fileMeta.id]: {
                    uploading: false,
                    percentage: 0,
                    totalChunks: 0,
                    totalChunksUploaded: 0
                }
            };
        });

        this.store.fileMetas.forEach((fileMeta) => {
            this.store.downloadStates = {
                ...this.store.downloadStates,
                [fileMeta.id]: {
                    downloading: false,
                    percentage: 0,
                    totalChunks: 0,
                    totalChunksDownloaded: 0
                }
            };
        });
    }

    async uploadFileHandler(e: InputEvent) {
        const files = (e.target as HTMLInputElement).files;
        const file = files?.[0];

        if (file === undefined) {
            return;
        }

        this.store.creatingFileMeta = true;

        const {
            fileId,
            bytes,
            chunkInfos
        } = await createFileMeta(file);

        this.store.creatingFileMeta = false;

        await this.fetchAndSetFileMetas();

        this.store.uploadStates = {
            ...this.store.uploadStates,
            [fileId]: {
                ...this.store.uploadStates[fileId],
                uploading: true,
                totalChunks: chunkInfos.length
            }
        };

        await uploadFile(
            fileId,
            bytes,
            chunkInfos,
            this.uploadFileNotifier.bind(this)
        );

        this.store.uploadStates = {
            ...this.store.uploadStates,
            [fileId]: {
                ...this.store.uploadStates[fileId],
                uploading: false
            }
        };
    }

    uploadFileNotifier(fileId: string) {
        const uploadState = this.store.uploadStates[fileId];

        const percentage = uploadState.totalChunks === 0 ? 0 : Math.floor((uploadState.totalChunksUploaded / uploadState.totalChunks) * 100);

        this.store.uploadStates = {
            ...this.store.uploadStates,
            [fileId]: {
                ...uploadState,
                percentage,
                totalChunksUploaded: uploadState.totalChunksUploaded + 1 
            }
        };
    }

    async downloadFileHandler(fileMeta: FileMeta) {
        this.store.downloadStates = {
            ...this.store.downloadStates,
            [fileMeta.id]: {
                downloading: true,
                percentage: 0,
                totalChunks: fileMeta.numChunks,
                totalChunksDownloaded: 0
            }
        };

        const allFileBytesArray = await fetchAllFileBytes(
            fileMeta,
            this.downloadFileNotifier.bind(this)
        );
        const allFileBytesUint8Array = Uint8Array.from(allFileBytesArray);

        const fileBlob = new Blob([allFileBytesUint8Array]);
        const fileBlobUrl = window.URL.createObjectURL(fileBlob);

        const a = document.createElement('a');

        a.href = fileBlobUrl;
        a.download = fileMeta.name;

        document.body.appendChild(a);

        a.click();

        window.URL.revokeObjectURL(fileBlobUrl);

        document.body.removeChild(a);

        this.store.downloadStates = {
            ...this.store.downloadStates,
            [fileMeta.id]: {
                ...this.store.downloadStates[fileMeta.id],
                downloading: false
            }
        };
    }

    downloadFileNotifier(fileId: string) {
        const downloadState = this.store.downloadStates[fileId];

        const percentage = downloadState.totalChunks === 0 ? 0 : Math.floor((downloadState.totalChunksDownloaded / downloadState.totalChunks) * 100);

        this.store.downloadStates = {
            ...this.store.downloadStates,
            [fileId]: {
                ...downloadState,
                percentage,
                totalChunksDownloaded: downloadState.totalChunksDownloaded + 1
            }
        };
    }

    render(state: State) {
        return html`
            <style>
                .file-info-container {
                    padding-bottom: 1rem;
                }

                .file-info {
                    padding: .5rem;
                }
            </style>

            <h1>Files</h1>

            <div>
                <input
                    type="file"
                    @change=${(e: InputEvent) => this.uploadFileHandler(e)}
                    .disabled=${state.creatingFileMeta === true}
                >
                <span ?hidden=${state.creatingFileMeta === false}>Saving...</span>
            </div>

            <br>

            <div>
                ${state.fileMetas.map((fileMeta) => {
                    const uploadState = state.uploadStates[fileMeta.id];
                    const downloadState = state.downloadStates[fileMeta.id];

                    if (
                        uploadState === undefined ||
                        downloadState === undefined
                    ) {
                        return html``;
                    }

                    return html`
                        <div class="file-info-container">
                            <div class="file-info">
                                ${fileMeta.name}
                                <button
                                    ?hidden=${uploadState.uploading === true || downloadState.downloading === true}
                                    @click=${() => this.downloadFileHandler(fileMeta)}
                                >
                                    Download
                                </button>
                                <div
                                    ?hidden=${uploadState.uploading === false}
                                >
                                    Uploading: ${uploadState.percentage}%
                                </div>
                                <div
                                    ?hidden=${downloadState.downloading === false}
                                >
                                    Downloading: ${downloadState.percentage}%
                                </div>
                            </div>
                        </div>
                    `;
                })}
            </div>
        `;
    }
}

window.customElements.define('files-app', FilesApp);

async function fetchFileMetas(): Promise<ReadonlyArray<FileMeta>> {
    const result = await query(gql`
        query {
            readFile {
                id
                name
                numChunks
            }
        }
    `);

    const fileMetas: ReadonlyArray<FileMeta> = result.data.readFile;

    return fileMetas;
}

async function fetchAllFileBytes(
    fileMeta: FileMeta,
    notifier: (fileId: string) => void,
    limit: number = 1
): Promise<ReadonlyArray<number>> {
    const promises = new Array(fileMeta.numChunks).fill(0).map(async (_, index) => {   
        const result = await query(gql`
            query (
                $fileId: ID!
                $offset: Int!
                $limit: Int!
            ) {
                readFileChunk(search: {
                    file: {
                        id: {
                            eq: $fileId
                        }
                    }
                }, limit: $limit, offset: $offset, order: {
                    startByte: ASC
                }) {
                    id
                    startByte
                    endByte
                }
            }
        `, {
            fileId: fileMeta.id,
            offset: index,
            limit
        });

        console.log('fetchAllFileBytes result', result);

        notifier(fileMeta.id);
    
        // TODO add error handling
        const bytes: ReadonlyArray<number> = result.data.readFileChunk[0].bytes;

        return bytes;
    });

    const byteArrays = await Promise.all(promises);

    return byteArrays.flat();
}

async function createFileMeta(
    file: File,
    limit: number = 500000 // TODO make a global setting for this that the user can configure
): Promise<{
    fileId: string;
    bytes: Uint8Array,
    chunkInfos: ReadonlyArray<ChunkInfo>
}> {
    const bytes = new Uint8Array(await file.arrayBuffer());
    
    const chunkInfos = getChunkInfos(
        0,
        limit,
        bytes.length
    );

    const fileId = await createFile(
        new Date(),
        file.name,
        chunkInfos.length
    );

    return {
        fileId,
        bytes,
        chunkInfos
    };
}

function getChunkInfos(
    offset: number,
    limit: number,
    totalBytes: number,
    chunkInfos: ReadonlyArray<ChunkInfo> = []
): ReadonlyArray<ChunkInfo> {
    if (limit + offset >= totalBytes) {
        return [
            ...chunkInfos, {
                startByte: offset,
                endByte: totalBytes - 1
            }
        ];
    }

    return getChunkInfos(
        offset + limit,
        limit,
        totalBytes, [
            ...chunkInfos, {
                startByte: offset,
                endByte: offset + limit - 1
            }
        ]
    );
}

async function createFile(
    createdAt: Date,
    name: string,
    numChunks: number
): Promise<string> {
    const createFileResult = await mutation(gql`
        mutation (
            $createdAt: Date!
            $fileType: FileType!
            $name: String!
            $numChunks: Int!
        ) {
            createFile(input: {
                createdAt: $createdAt
                name: $name
                numChunks: $numChunks
            }) {
                id
            }
        }
    `, {
        createdAt,
        name,
        numChunks
    });

    console.log('createFileResult', createFileResult);

    // TODO add error handling
    const fileId: string = createFileResult.data.createFile[0].id;

    return fileId;
}

async function uploadFile(
    fileId: string,
    bytes: Uint8Array,
    chunkInfos: ReadonlyArray<ChunkInfo>,
    notifier: (fileId: string) => void
): Promise<void> {
    const promises = chunkInfos.map(async (chunkInfo) => {
        const slice = Array.from(bytes.slice(chunkInfo.startByte, chunkInfo.endByte + 1));

        await createFileChunk(
            slice,
            chunkInfo.endByte,
            fileId,
            chunkInfo.startByte
        );        

        notifier(fileId);
    });

    await Promise.all(promises);
}

async function createFileChunk(
    bytes: ReadonlyArray<number>,
    endByte: number,
    fileId: string,
    startByte: number
): Promise<void> {
    const createFileChunkResult = await mutation(gql`
        mutation (
            $bytes: Blob!
            $endByte: Int!
            $fileId: ID!
            $startByte: Int!
        ) {
            createFileChunk(input: {
                bytes: $bytes
                endByte: $endByte
                file: {
                    connect: $fileId
                }
                startByte: $startByte
            }) {
                id
            }
        }
    `, {
        bytes,
        endByte,
        fileId,
        startByte
    });

    console.log('createFileChunkResult', createFileChunkResult);
}