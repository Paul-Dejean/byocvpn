import type { PermissionStatus } from "../bindings";

export type { PermissionStatus };

export type Permissions = PermissionStatus[];

export function areAllPermissionsGranted(permissions: Permissions): boolean {
  return permissions.every((status) => status.granted);
}
