export function getEnvironment(): 'production' | 'development' {
    if (window.location.origin.endsWith('ic0.app')) {
        return 'production';
    }

    return 'development';
}

export const GRAPHQL_CANISTER_ID = {
    production: 'ix47w-raaaa-aaaae-qaalq-cai',
    development: 'ryjl3-tyaaa-aaaaa-aaaba-cai'
}[getEnvironment()];