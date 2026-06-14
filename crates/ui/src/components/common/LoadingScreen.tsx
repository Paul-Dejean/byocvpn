import { Spinner } from "../primitives/Spinner";

interface LoadingScreenProps {
  message?: string;
}

export function LoadingScreen({ message = "Loading..." }: LoadingScreenProps) {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <Spinner size="w-16 h-16" color="border-blue-500" />
        <p className="text-gray-300 mt-4">{message}</p>
      </div>
    </div>
  );
}
