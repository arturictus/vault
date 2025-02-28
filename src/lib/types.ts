export interface Secret {
    id: string;
    name: string;
    value: string;
}

export type Secrets = Secret[];