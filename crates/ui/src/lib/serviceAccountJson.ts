export function readProjectIdFromServiceAccountJson(serviceAccountJson: string): string | null {
  try {
    const parsed: unknown = JSON.parse(serviceAccountJson);
    if (
      typeof parsed === "object" &&
      parsed !== null &&
      "project_id" in parsed &&
      typeof parsed.project_id === "string"
    ) {
      return parsed.project_id;
    }
    return null;
  } catch {
    return null;
  }
}
