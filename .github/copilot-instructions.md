# GitHub Copilot Instructions for On-Demand VPN

## Project Overview

This is an On-Demand VPN desktop application built with Rust (Tauri) and React (TypeScript). It manages ephemeral AWS EC2-based WireGuard VPN servers.

## Code Style Guidelines

### General Rules

- **No abbreviations in variable names**: Use `region` not `rgn`, `instance` not `inst`, `configuration` not `cfg`
- **Descriptive names**: All variables, functions, and types should have clear, descriptive names
- **One component per file**: Each React component should be in its own file

### Rust Guidelines

- Always import types at the top of the file, never use fully qualified names in code
  - ✅ Good: `use std::collections::HashMap;` then use `HashMap`
  - ❌ Bad: Use `std::collections::HashMap` directly in code
- Use `Result<T, E>` for error handling, avoid panicking
- Use `async/await` for I/O operations
- Follow rustfmt and clippy recommendations

### TypeScript/React Guidelines

- Use TypeScript strict mode
- Define interfaces for all component props
- Use functional components with hooks
- Event handlers should follow the pattern: `on[Action]` (e.g., `onSelectInstance`, `onDisconnect`)
- Avoid `any` types - use proper interfaces
- Keep components under 200 lines - split larger components
- Use hooks in the component that needs them (avoid prop drilling)
- **Use enums instead of raw strings when possible** for better type safety
  - ✅ Good: `enum Page { LANDING = "LANDING", VPN = "VPN" }` then use `Page.VPN`
  - ❌ Bad: `const page = "vpn"` with string literals everywhere

### Naming Conventions

- **React Components**: PascalCase (`ServerList`, `RegionSelector`)
- **Functions/Variables**: camelCase (`handleDeploy`, `selectedRegion`)
- **Rust Types**: PascalCase (`AwsRegion`, `ExistingInstance`)
- **Rust Functions**: snake_case (`spawn_server`, `terminate_instance`)
- **Constants**: SCREAMING_SNAKE_CASE (`DEFAULT_TIMEOUT`)
- **Files**: Match component/module name (`ServerList.tsx`, `ec2.rs`)

## Architecture

### Crate Structure

```
crates/
├── ui/          # Tauri desktop app with React frontend
├── core/        # Core business logic and VPN management
├── daemon/      # Background service for VPN connections
├── cli/         # Command-line interface
└── infra/aws/   # AWS infrastructure operations
```

### Component Organization

```
crates/ui/src/components/
├── common/      # Shared UI components (LoadingSpinner, EmptyState, Toast)
├── servers/     # Server management (ServerList, ServerCard, ServerDetails)
├── regions/     # Region selection (RegionSelector)
├── settings/    # Settings UI
└── vpn/         # Main views (ServerManagementView, ConnectedView)
```

### React Patterns

#### Hook Usage

- Components should use hooks directly for data they need
- Don't pass too many props through parent components
- Example: `ServerManagementView` uses `useRegions`, `useInstances`, etc. directly

#### Component Responsibilities

- **VpnPage**: Simple router between views based on connection status
- **ServerManagementView**: Manages all server operations (spawn, terminate, list)
- **ConnectedView**: Manages VPN metrics and disconnect when connected
- **ServerList**: Displays list of servers with add button
- **ServerDetails**: Shows details and actions for selected server
- **RegionSelector**: Full-screen view for region selection

#### Props Interface Documentation

Always document props with JSDoc:

```typescript
/**
 * Props for the ServerCard component
 */
interface ServerCardProps {
  /** The server instance to display */
  instance: ExistingInstance;
  /** Whether this card is currently selected */
  isSelected: boolean;
  /** Callback when the card is clicked */
  onSelect: (instance: ExistingInstance) => void;
}
```

### Error Handling

- Display user-friendly error messages (non-technical language)
- Log detailed errors for debugging
- Show loading states during all async operations
- Provide fallback UI for error states

### Testing

- Write unit tests alongside implementation
- Test error cases and edge cases
- Mock AWS API calls in tests
- Use descriptive test names

## Common Patterns

### Tauri Command Handler

```rust
#[tauri::command]
async fn spawn_server(
    state: State<'_, AppState>,
    region: String,
) -> Result<ServerInfo, String> {
    let server_manager = state.server_manager.lock().await;

    server_manager
        .spawn_server(&region)
        .await
        .map_err(|e| e.to_string())
}
```

### React Component with Hooks

```typescript
export function ServerCard({
  instance,
  isSelected,
  onSelect,
}: ServerCardProps) {
  // Component logic here
  return (
    <button onClick={() => onSelect(instance)}>
      {/* UI here */}
    </button>
  );
}
```

### Async Error Handling (Rust)

```rust
pub async fn spawn_server(&self, region: &str) -> Result<ServerInfo> {
    // Do async work
    let instance = self.ec2_client.launch_instance(config).await?;

    // Return result
    Ok(server_info)
}
```

## Common Tasks

### Adding a New React Component

1. Create file in appropriate directory
2. Define props interface with JSDoc
3. Export function component
4. Keep under 200 lines
5. Add to parent component imports

### Adding a Tauri Command

1. Add function to `crates/ui/src-tauri/src/commands/`
2. Mark with `#[tauri::command]`
3. Return `Result<T, String>` for errors
4. Register in `main.rs`
5. Call from React with `invoke('command_name', { args })`

### Adding a New Hook

1. Create in `crates/ui/src/hooks/`
2. Export from `hooks/index.ts`
3. Use in component that needs the data
4. Return state, loading flag, error, and handler functions

## What NOT to Do

❌ Don't use abbreviated variable names (`rgn`, `inst`, `cfg`)
❌ Don't use fully qualified names in Rust code
❌ Don't use `any` type in TypeScript
❌ Don't use raw string literals when enums are appropriate
❌ Don't create components over 200 lines
❌ Don't pass 10+ props to components (use hooks instead)
❌ Don't panic in Rust code (use `Result`)
❌ Don't show technical errors to users
❌ Don't forget loading states for async operations

## Resources

- [Project Constitution](.github/prompts/speckit.constitution.prompt.md)
- [Project Specification](.github/prompts/speckit.specify.prompt.md)
- [Implementation Guide](.github/prompts/speckit.implement.prompt.md)
