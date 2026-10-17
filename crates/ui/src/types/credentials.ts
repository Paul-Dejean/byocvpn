import type {
  AwsCredentials,
  AzureCredentials,
  GcpCredentials,
  OracleCredentials,
  ProviderCredentials,
} from "../bindings";
import { CloudProviderName } from "./providers";

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

export function buildProviderCredentials(
  request: VerifiableCredentialsRequest,
): ProviderCredentials {
  switch (request.provider) {
    case CloudProviderName.Aws:
      return { provider: request.provider, ...request.credentials };
    case CloudProviderName.Gcp:
      return { provider: request.provider, ...request.credentials };
    case CloudProviderName.Azure:
      return { provider: request.provider, ...request.credentials };
    default:
      throw new Error("Unsupported provider for verifiable credentials");
  }
}
