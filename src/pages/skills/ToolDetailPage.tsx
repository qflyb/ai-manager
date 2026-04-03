import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { Fragment, useEffect, useState } from "react";
import { useAiTools } from "../../hooks/useAiTools";
import { useSkills } from "../../hooks/useSkills";
import { useCommands } from "../../hooks/useCommands";
import SkillCard from "../../components/skills/SkillCard";
import SkillActions from "../../components/skills/SkillActions";
import CommandsViewer from "../../components/skills/CommandsViewer";
import ConfigViewer from "../../components/skills/ConfigViewer";
import EmptyState from "../../components/ui/EmptyState";
import LoadingSpinner from "../../components/ui/LoadingSpinner";
import Badge from "../../components/ui/Badge";
import SearchInput from "../../components/ui/SearchInput";

export default function ToolDetailPage() {
  const { toolId } = useParams<{ toolId: string }>();
  const navigate = useNavigate();
  const { tools } = useAiTools();
  const { skills, loading, error, refetch } = useSkills(toolId);
  const tool = tools.find((t) => t.id === toolId);
  const hasCommands = Boolean(tool?.commands_dir);
  const {
    commands,
    loading: commandsLoading,
    error: commandsError,
    refetch: refetchCommands,
  } = useCommands(hasCommands ? toolId : undefined);
  const [search, setSearch] = useState("");
  const [tab, setTab] = useState<"skills" | "commands" | "config">("skills");

  useEffect(() => {
    if (!hasCommands && tab === "commands") {
      setTab("skills");
    }
  }, [hasCommands, tab]);

  const filtered = skills.filter(
    (s) =>
      s.name.toLowerCase().includes(search.toLowerCase()) ||
      s.description.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="p-8">
      <button
        onClick={() => navigate("/skills")}
        className="mb-4 flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to tools
      </button>

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">
            {tool?.name || toolId}
          </h1>
          {tool && (
            <p className="mt-1 text-xs text-gray-400">{tool.config_dir}</p>
          )}
        </div>
        <Badge variant="success">{skills.length} skills</Badge>
      </div>

      {/* Tabs */}
      <div className="mt-6 flex gap-1 border-b border-gray-200">
        <button
          onClick={() => setTab("skills")}
          className={`border-b-2 px-4 py-2.5 text-sm font-medium transition-colors ${
            tab === "skills"
              ? "border-indigo-500 text-indigo-600"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          Skills
        </button>
        {hasCommands && (
          <button
            onClick={() => setTab("commands")}
            className={`border-b-2 px-4 py-2.5 text-sm font-medium transition-colors ${
              tab === "commands"
                ? "border-indigo-500 text-indigo-600"
                : "border-transparent text-gray-500 hover:text-gray-700"
            }`}
          >
            Commands
          </button>
        )}
        <button
          onClick={() => setTab("config")}
          className={`border-b-2 px-4 py-2.5 text-sm font-medium transition-colors ${
            tab === "config"
              ? "border-indigo-500 text-indigo-600"
              : "border-transparent text-gray-500 hover:text-gray-700"
          }`}
        >
          Config Files
        </button>
      </div>

      {tab === "skills" && (
        <div className="mt-6">
          <div className="mb-4 w-64">
            <SearchInput
              value={search}
              onChange={setSearch}
              placeholder="Search skills..."
            />
          </div>

          {loading && <LoadingSpinner text="Loading skills..." />}
          {error && (
            <div className="rounded-lg bg-red-50 p-4 text-sm text-red-600">
              {error}
            </div>
          )}

          {!loading && filtered.length === 0 && (
            <EmptyState
              title="No skills installed"
              description="Install skills from the Skills Hub"
            />
          )}

          <div className="grid grid-cols-[1fr_auto] items-start gap-x-2 gap-y-3">
            {filtered.map((skill) => (
              <Fragment key={skill.dir_name}>
                <div className={skill.disabled ? "opacity-50" : ""}>
                  <SkillCard skill={skill} toolId={toolId} />
                </div>
                <div className={`self-start pt-3 ${skill.disabled ? "opacity-50" : ""}`}>
                  <SkillActions
                    toolId={toolId!}
                    skillName={skill.dir_name}
                    isSymlink={skill.is_symlink}
                    disabled={skill.disabled}
                    onComplete={refetch}
                  />
                </div>
              </Fragment>
            ))}
          </div>
        </div>
      )}

      {tab === "commands" && toolId && hasCommands && (
        <div className="mt-6">
          <CommandsViewer
            toolId={toolId}
            commands={commands}
            loading={commandsLoading}
            error={commandsError}
            onRefresh={refetchCommands}
          />
        </div>
      )}

      {tab === "config" && tool && (
        <div className="mt-6">
          <ConfigViewer configFiles={tool.config_files} />
        </div>
      )}
    </div>
  );
}
