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

export type BlogPostInputTexts = {
    [userId: string]: string;
};

export type BlogPostsAreSaving = {
    [userId: string]: boolean;
};

export interface UsernameInputTextChangedEvent extends CustomEvent {
    detail: Readonly<{
        usernameInputText: string;
    }>;
}

export interface CreateBlogPostEvent extends CustomEvent {
    detail: Readonly<{
        userId: string;
    }>;
}

export interface BlogPostInputTextChangedEvent extends CustomEvent {
    detail: Readonly<{
        userId: string;
        blogPostInputText: string;
    }>;
}