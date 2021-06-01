export type User = {
    id: string;
    username: string;
    blogPosts: ReadonlyArray<BlogPost>;
};

export type BlogPost = {
    id: string;
    publishedAt?: Date;
    title: string;
    author: User;
};