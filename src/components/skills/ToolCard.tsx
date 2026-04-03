import { useNavigate } from "react-router-dom";
import { FolderOpen, ChevronRight } from "lucide-react";
import type { AiTool } from "../../types/skills";
import Badge from "../ui/Badge";
import StatusDot from "../ui/StatusDot";

interface ToolCardProps {
  tool: AiTool;
}

const capabilityBadge = {
  skills: { label: "Skills", variant: "success" as const },
  "config-only": { label: "Config Only", variant: "warning" as const },
  "detected-only": { label: "Detected", variant: "muted" as const },
};

export default function ToolCard({ tool }: ToolCardProps) {
  const navigate = useNavigate();
  const badge = capabilityBadge[tool.capability] || capabilityBadge["detected-only"];

  return (
    <button
      onClick={() =>
        tool.capability === "skills" && navigate(`/skills/tools/${tool.id}`)
      }
      disabled={tool.capability !== "skills"}
      className={`group flex flex-col items-start rounded-xl border border-gray-200 bg-white p-5 text-left transition-all ${
        tool.capability === "skills"
          ? "cursor-pointer hover:border-indigo-200 hover:shadow-md"
          : "cursor-default opacity-70"
      }`}
    >
      <div className="flex w-full items-center justify-between">
        <div className="flex items-center gap-2">
          <StatusDot status={tool.detected ? "active" : "inactive"} />
          <span className="text-sm font-semibold text-gray-900">
            {tool.name}
          </span>
        </div>
        {tool.capability === "skills" && (
          <ChevronRight className="h-4 w-4 text-gray-300 transition-colors group-hover:text-indigo-400" />
        )}
      </div>

      <div className="mt-3 flex items-center gap-2">
        <Badge variant={badge.variant}>{badge.label}</Badge>
        {tool.capability === "skills" && tool.skill_count > 0 && (
          <Badge variant="info">{tool.skill_count} skills</Badge>
        )}
      </div>

      <div className="mt-3 flex w-full items-center gap-1 overflow-hidden text-xs text-gray-400">
        <FolderOpen className="h-3 w-3" />
        <span className="truncate">{tool.config_dir}</span>
      </div>
    </button>
  );
}
