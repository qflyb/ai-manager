import { useState, useEffect, useRef } from "react";
import { FileCode, ExternalLink, ChevronDown, Check } from "lucide-react";
import type { ConfigFile } from "../../types/skills";
import { useEditorPreference } from "../../hooks/useEditorPreference";
import * as api from "../../api/skills";

interface ConfigViewerProps {
  configFiles: ConfigFile[];
}

export default function ConfigViewer({ configFiles }: ConfigViewerProps) {
  const [selected, setSelected] = useState<ConfigFile | null>(
    configFiles[0] || null
  );
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [opening, setOpening] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { editors, editor, setEditor, loading: editorsLoading } =
    useEditorPreference();

  useEffect(() => {
    if (!selected) return;
    setLoading(true);
    api
      .readConfigFile(selected.path)
      .then(setContent)
      .catch((e) => setContent(`Error: ${e}`))
      .finally(() => setLoading(false));
  }, [selected]);

  useEffect(() => {
    if (!dropdownOpen) return;
    function handleClickOutside(e: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node)
      ) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownOpen]);

  const currentEditor = editors.find((e) => e.id === editor);

  const handleOpenInEditor = async () => {
    if (!selected || !editor) return;
    setOpening(true);
    try {
      await api.openInEditor(selected.path, editor);
    } catch (e) {
      alert(String(e));
    } finally {
      setOpening(false);
    }
  };

  if (configFiles.length === 0) {
    return (
      <p className="text-sm text-gray-400">No config files found.</p>
    );
  }

  const showEditorButton = !editorsLoading && editors.length > 0;

  return (
    <div>
      <div className="flex items-center justify-between border-b border-gray-200">
        <div className="flex gap-1">
          {configFiles.map((f) => (
            <button
              key={f.path}
              onClick={() => setSelected(f)}
              className={`flex items-center gap-1.5 border-b-2 px-3 py-2 text-xs font-medium transition-colors ${
                selected?.path === f.path
                  ? "border-indigo-500 text-indigo-600"
                  : "border-transparent text-gray-500 hover:text-gray-700"
              }`}
            >
              <FileCode className="h-3 w-3" />
              {f.name}
            </button>
          ))}
        </div>

        {showEditorButton && (
          <div className="relative mb-1" ref={dropdownRef}>
            <div className="flex items-stretch">
              <button
                onClick={handleOpenInEditor}
                disabled={!selected || opening}
                className="flex items-center gap-1.5 rounded-l-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-700 disabled:opacity-50"
              >
                <ExternalLink className="h-3 w-3" />
                {opening
                  ? "Opening..."
                  : `Open in ${currentEditor?.label ?? editor}`}
              </button>
              <button
                onClick={() => setDropdownOpen((prev) => !prev)}
                className="flex items-center rounded-r-md border-l border-indigo-500 bg-indigo-600 px-1.5 text-white hover:bg-indigo-700"
              >
                <ChevronDown className="h-3 w-3" />
              </button>
            </div>

            {dropdownOpen && (
              <div className="absolute right-0 z-10 mt-1 w-44 rounded-lg border border-gray-200 bg-white py-1 shadow-lg">
                {editors.map((opt) => (
                  <button
                    key={opt.id}
                    onClick={() => {
                      setEditor(opt.id);
                      setDropdownOpen(false);
                    }}
                    className="flex w-full items-center justify-between px-3 py-2 text-xs text-gray-700 hover:bg-gray-50"
                  >
                    {opt.label}
                    {editor === opt.id && (
                      <Check className="h-3 w-3 text-indigo-600" />
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="mt-3 max-h-[400px] overflow-auto rounded-lg bg-gray-900 p-4">
        {loading ? (
          <p className="text-xs text-gray-500">Loading...</p>
        ) : (
          <pre className="text-xs leading-relaxed text-gray-100 whitespace-pre-wrap">
            {content}
          </pre>
        )}
      </div>
    </div>
  );
}
