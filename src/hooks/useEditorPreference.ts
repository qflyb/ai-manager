import { useState, useEffect, useCallback } from "react";
import type { EditorInfo } from "../types/skills";
import * as api from "../api/skills";

const STORAGE_KEY = "ai-manager:preferred-editor";

export function useEditorPreference() {
  const [editors, setEditors] = useState<EditorInfo[]>([]);
  const [editor, setEditorState] = useState<string>(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) || "";
    } catch {
      return "";
    }
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api
      .detectEditors()
      .then((detected) => {
        setEditors(detected);
        setEditorState((prev) => {
          const valid = detected.find((e) => e.id === prev);
          const resolved = valid ? prev : detected[0]?.id || "";
          try {
            if (resolved) localStorage.setItem(STORAGE_KEY, resolved);
          } catch {
            // ignore
          }
          return resolved;
        });
      })
      .catch(() => setEditors([]))
      .finally(() => setLoading(false));
  }, []);

  const setEditor = useCallback((id: string) => {
    setEditorState(id);
    try {
      localStorage.setItem(STORAGE_KEY, id);
    } catch {
      // ignore
    }
  }, []);

  return { editors, editor, setEditor, loading };
}
