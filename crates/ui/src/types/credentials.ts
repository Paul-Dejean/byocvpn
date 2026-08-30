import type {
  AwsCredentials,
  AzureCredentials,
  GcpCredentials,
  OracleCredentials,
  ProviderCredentials,
} from "../bindings";

export type {
  AwsCredentials,
  AzureCredentials,
  GcpCredentials,
  OracleCredentials,
  ProviderCredentials,
};

export type VerifiableCredentialsMap = {
  AWS: AwsCredentials;
  GCP: GcpCredentials;
  AZURE: AzureCredentials;
};

export type VerifiableProvider = keyof VerifiableCredentialsMap;

export type VerifiableCredentials = VerifiableCredentialsMap[VerifiableProvider];

export type VerifiableCredentialsRequest = {
  [Provider in VerifiableProvider]: {
    provider: Provider;
    credentials: VerifiableCredentialsMap[Provider];
  };
}[VerifiableProvider];
