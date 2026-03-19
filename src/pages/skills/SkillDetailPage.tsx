import { useParams, useNavigate, useLocation } from "react-router-dom";
import { ArrowLeft, Link2, FolderOpen } from "lucide-react";
import { useSkillContent } from "../../hooks/useSkillContent";
import { useSkills } from "../../hooks/useSkills";
import { useSkillsStore } from "../../store/useSkillsStore";
import MarkdownViewer from "../../components/ui/MarkdownViewer";
import LoadingSpinner from "../../components/ui/LoadingSpinner";
import Badge from "../../components/ui/Badge";
import { useMemo } from "react";

export default function SkillDetailPage() {
  const { toolId, skillName } = useParams<{
    toolId?: string;
    skillName: string;
  }>();
  const navigate = useNavigate();
  const location = useLocation();
  const isHub = location.pathname.startsWith("/skills/hub");

  // Get skill path from either tool skills or hub
  const { skills: toolSkills } = useSkills(toolId);
  const { hubSkills } = useSkillsStore();

  const skill = useMemo(() => {
    if (isHub) {
      return hubSkills.find((s) => s.dir_name === skillName);
    }
    return toolSkills.find((s) => s.dir_name === skillName);
  }, [isHub, hubSkills, toolSkills, skillName]);

  const { content, loading, error } = useSkillContent(skill?.dir_path);

  const backPath = isHub
    ? "/skills/hub"
    : toolId
      ? `/skills/tools/${toolId}`
      : "/skills";

  return (
    <div>
      <div className="sticky top-0 z-10 bg-white/80 backdrop-blur-sm border-b border-gray-100 px-8 py-3">
        <button
          onClick={() => navigate(backPath)}
          className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>
      </div>
      <div className="p-8">

      {loading && <LoadingSpinner text="Loading skill content..." />}
      {error && (
        <div className="rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {error}
        </div>
      )}

      {content && skill && (
        <div className="flex gap-8">
          {/* Main content */}
          <div className="min-w-0 flex-1">
            <h1 className="text-2xl font-bold text-gray-900">
              {content.frontmatter.name || skillName}
            </h1>

            {content.frontmatter.description && (
              <p className="mt-2 text-sm text-gray-600">
                {content.frontmatter.description}
              </p>
            )}

            <div className="mt-2 flex flex-wrap gap-2">
              {content.frontmatter["allowed-tools"] && (
                <Badge variant="info">
                  Tools: {content.frontmatter["allowed-tools"]}
                </Badge>
              )}
              {skill.is_symlink && (
                <Badge variant="warning">
                  <Link2 className="mr-1 inline h-3 w-3" />
                  Symlink
                </Badge>
              )}
            </div>

            <div className="mt-6 rounded-xl border border-gray-200 bg-white p-6">
              <MarkdownViewer content={content.markdown_body} />
            </div>

            {/* Reference files */}
            {content.references.length > 0 && (
              <div className="mt-6">
                <h2 className="mb-3 text-sm font-semibold text-gray-600 uppercase tracking-wide">
                  Reference Files
                </h2>
                <div className="space-y-3">
                  {content.references.map((ref) => (
                    <details
                      key={ref.name}
                      className="rounded-lg border border-gray-200 bg-white"
                    >
                      <summary className="flex cursor-pointer items-center gap-2 px-4 py-3 text-sm font-medium text-gray-700 hover:text-gray-900">
                        <FolderOpen className="h-4 w-4 text-gray-400" />
                        {ref.name}
                      </summary>
                      <div className="border-t border-gray-100 p-4">
                        <MarkdownViewer content={ref.content} />
                      </div>
                    </details>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Side panel */}
          <div className="w-56 shrink-0">
            <div className="rounded-xl border border-gray-200 bg-white p-4">
              <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                Info
              </h3>
              <dl className="mt-3 space-y-3 text-sm">
                <div>
                  <dt className="text-xs text-gray-400">Directory</dt>
                  <dd className="mt-0.5 truncate text-xs text-gray-600">
                    {skill.dir_name}
                  </dd>
                </div>
                {skill.symlink_target && (
                  <div>
                    <dt className="text-xs text-gray-400">Symlink Target</dt>
                    <dd className="mt-0.5 truncate text-xs text-gray-600">
                      {skill.symlink_target}
                    </dd>
                  </div>
                )}
                {skill.installed_in.length > 0 && (
                  <div>
                    <dt className="text-xs text-gray-400">Installed In</dt>
                    <dd className="mt-1 flex flex-wrap gap-1">
                      {skill.installed_in.map((t) => (
                        <Badge key={t} variant="info">
                          {t}
                        </Badge>
                      ))}
                    </dd>
                  </div>
                )}
                <div>
                  <dt className="text-xs text-gray-400">Contents</dt>
                  <dd className="mt-1 flex flex-wrap gap-1">
                    <Badge variant="muted">SKILL.md</Badge>
                    {skill.has_references && (
                      <Badge variant="muted">references/</Badge>
                    )}
                    {skill.has_agents && (
                      <Badge variant="muted">agents/</Badge>
                    )}
                    {skill.has_scripts && (
                      <Badge variant="muted">scripts/</Badge>
                    )}
                  </dd>
                </div>
              </dl>
            </div>
          </div>
        </div>
      )}
      </div>
    </div>
  );
}
