import { useNavigate } from "react-router-dom";
import { Puzzle, Github, FolderOpen, ChevronRight } from "lucide-react";
import type { PluginEntry } from "../../types/plugins";
import Badge from "../ui/Badge";

interface PluginCardProps {
  plugin: PluginEntry;
  skillCount?: number;
  commandCount?: number;
}

export default function PluginCard({
  plugin,
  skillCount,
  commandCount,
}: PluginCardProps) {
  const navigate = useNavigate();
  const isGithub = plugin.source.type === "GitHub";

  return (
    <button
      onClick={() => navigate(`/plugins/${plugin.id}`)}
      className="group flex w-full items-start gap-3 rounded-lg border border-gray-200 bg-white p-4 text-left transition-all hover:border-violet-200 hover:shadow-sm"
    >
      <div className="mt-0.5 rounded-lg bg-violet-50 p-2">
        <Puzzle className="h-4 w-4 text-violet-500" />
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-gray-900">
            {plugin.metadata.name}
          </span>
          {plugin.metadata.version && (
            <span className="text-xs text-gray-400">
              v{plugin.metadata.version}
            </span>
          )}
        </div>

        {plugin.metadata.description && (
          <p className="mt-1 line-clamp-2 text-xs text-gray-500">
            {plugin.metadata.description}
          </p>
        )}

        <div className="mt-2 flex flex-wrap gap-1.5">
          <Badge variant={isGithub ? "info" : "muted"}>
            {isGithub ? (
              <span className="flex items-center gap-1">
                <Github className="h-2.5 w-2.5" />
                {plugin.source.type === "GitHub" &&
                  `${plugin.source.owner}/${plugin.source.repo}`}
              </span>
            ) : (
              <span className="flex items-center gap-1">
                <FolderOpen className="h-2.5 w-2.5" />
                Local
              </span>
            )}
          </Badge>
          {plugin.metadata.author && (
            <Badge variant="muted">{plugin.metadata.author}</Badge>
          )}
          {skillCount !== undefined && skillCount > 0 && (
            <Badge variant="success">
              {skillCount} skill{skillCount > 1 ? "s" : ""}
            </Badge>
          )}
          {commandCount !== undefined && commandCount > 0 && (
            <Badge variant="success">
              {commandCount} command{commandCount > 1 ? "s" : ""}
            </Badge>
          )}
        </div>
      </div>

      <ChevronRight className="mt-1 h-4 w-4 flex-shrink-0 text-gray-300 transition-colors group-hover:text-violet-400" />
    </button>
  );
}
