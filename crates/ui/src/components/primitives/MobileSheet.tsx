import { PointerEvent, ReactNode, useEffect, useRef, useState } from "react";

const DISMISS_DRAG_DISTANCE = 96;

interface MobileSheetProps {
  isOpen: boolean;
  onClose: () => void;
  isFullHeight?: boolean;
  children: ReactNode;
}

export function MobileSheet({
  isOpen,
  onClose,
  isFullHeight = false,
  children,
}: MobileSheetProps) {
  const [shouldRenderContent, setShouldRenderContent] = useState(isOpen);
  const [dragOffset, setDragOffset] = useState(0);
  const dragStartY = useRef<number | null>(null);

  useEffect(() => {
    if (isOpen) {
      setShouldRenderContent(true);
      setDragOffset(0);
    }
  }, [isOpen]);

  function onDragStart(event: PointerEvent<HTMLDivElement>) {
    dragStartY.current = event.clientY;
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function onDragMove(event: PointerEvent<HTMLDivElement>) {
    if (dragStartY.current === null) {
      return;
    }
    setDragOffset(Math.max(0, event.clientY - dragStartY.current));
  }

  function onDragEnd() {
    if (dragStartY.current === null) {
      return;
    }
    dragStartY.current = null;
    if (dragOffset > DISMISS_DRAG_DISTANCE) {
      onClose();
      return;
    }
    setDragOffset(0);
  }

  function onSheetTransitionEnd() {
    if (!isOpen) {
      setShouldRenderContent(false);
    }
  }

  const isDragging = dragStartY.current !== null;
  const sheetTransform = isOpen ? `translateY(${dragOffset}px)` : "translateY(100%)";

  return (
    <>
      <div
        className={`absolute inset-0 z-40 bg-overlay transition-opacity duration-300 ${
          isOpen ? "opacity-60 pointer-events-auto" : "opacity-0 pointer-events-none"
        }`}
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        aria-hidden={!isOpen}
        onTransitionEnd={onSheetTransitionEnd}
        style={{ transform: sheetTransform }}
        className={`absolute inset-x-0 bottom-0 z-50 flex flex-col rounded-t-2xl bg-bg-bolder border-t border-bd-moderate shadow-2xl ${
          isFullHeight ? "h-[92%]" : "max-h-[85%]"
        } ${isDragging ? "" : "transition-transform duration-300 ease-out"}`}
      >
        <div
          onPointerDown={onDragStart}
          onPointerMove={onDragMove}
          onPointerUp={onDragEnd}
          onPointerCancel={onDragEnd}
          className="flex-shrink-0 flex justify-center pt-2.5 pb-1.5 touch-none cursor-grab"
        >
          <div className="h-1 w-10 rounded-full bg-bd-strong" />
        </div>
        {shouldRenderContent && (
          <div className="flex-1 min-h-0 flex flex-col safe-area-bottom">{children}</div>
        )}
      </div>
    </>
  );
}
