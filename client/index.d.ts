import { Identity } from '@dfinity/agent';

export type Options = Readonly<{
    canisterId: string;
    identity?: Identity;
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