export type BlogPost = {
    id: string | symbol;
    body: string;
    created_at: ISOString;
    live: boolean;
    num_words: number;
    published_at: ISOString | null;
    title: string;
    updated_at: ISOString;
}

export type ISOString = string;