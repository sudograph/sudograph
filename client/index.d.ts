export type Options = Readonly<{
    canisterId: string;
    queryFunctionName?: string;
    mutationFunctionName?: string;
}>;

export type Variables = Readonly<{
    [key: string]: any;
}>;

export {
    gql,
    sudograph
} from './sudograph';