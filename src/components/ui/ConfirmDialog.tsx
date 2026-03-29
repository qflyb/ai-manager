interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  title,
  message,
  confirmLabel = "Confirm",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/40"
        onClick={onCancel}
      />
      <div className="relative w-full max-w-sm rounded-xl bg-white p-6 shadow-xl">
        <h2 className="text-base font-semibold text-gray-900">{title}</h2>
        <p className="mt-2 text-sm text-gray-600 whitespace-pre-line">{message}</p>
        <div className="mt-5 flex justify-end gap-2">
          <button
            onClick={onCancel}
            className="rounded-lg px-4 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-100"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
