import { AppError } from "../../types";
import { describeError } from "../../lib/describeError";
import { Banner } from "../primitives/Banner";

interface ErrorBannerProps {
  error: AppError;
  title?: string;
  onDismiss?: () => void;
  className?: string;
}

export function ErrorBanner({ error, title, onDismiss, className }: ErrorBannerProps) {
  const description = describeError(error);

  return (
    <Banner
      variant="danger"
      title={title ?? description.title}
      onDismiss={onDismiss}
      className={className}
    >
      <div className="flex flex-col gap-1">
        <p>{description.description}</p>
        {description.hint && <p>{description.hint}</p>}
        {description.detail && (
          <details className="text-caption text-fg-medium">
            <summary className="cursor-pointer select-none">Show details</summary>
            <pre className="mt-1 font-mono wrap-break-word whitespace-pre-wrap">
              {description.detail}
            </pre>
          </details>
        )}
      </div>
    </Banner>
  );
}
