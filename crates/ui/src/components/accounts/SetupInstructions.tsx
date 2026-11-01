import { ProviderSetupInstructions } from "../../constants/providerSetup";

interface SetupInstructionsProps {
  instructions: ProviderSetupInstructions;
}

export function SetupInstructions({ instructions }: SetupInstructionsProps) {
  return (
    <section className="flex flex-col gap-4 rounded-xl border border-bd-moderate bg-bg-bolder p-4">
      <h2 className="text-body-sm text-fg-lighter">{instructions.title}</h2>
      <ol className="flex flex-col gap-3">
        {instructions.steps.map((setupStep) => (
          <li key={setupStep.number} className="flex gap-3">
            <span className="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border border-bd-moderate bg-bg-medium text-caption text-fg-medium">
              {setupStep.number}
            </span>
            <p className="text-caption leading-relaxed text-fg-medium">{setupStep.text}</p>
          </li>
        ))}
      </ol>
    </section>
  );
}
