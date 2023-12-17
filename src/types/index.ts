export type Profile = {
  relay?: string;
  publicKey?: string;
  about?: string;
  lud06?: string;
  lud16?: string;
  name?: string;
  nip05?: string;
  picture?: string;
  website?: string;
  banner?: string;
  location?: string;
  github?: string;
  [key: string]: unknown;
};

export type BitcoinCoreConfig = {
  host: string;
  port: number;
  rpc_user: string;
  rpc_password: string;
};
