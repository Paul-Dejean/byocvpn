export interface PermissionStatus {
  permission: string;
  granted: boolean;
}

export type Permissions = PermissionStatus[];

export function areAllPermissionsGranted(permissions: Permissions): boolean {
  return permissions.every((status) => status.granted);
}
