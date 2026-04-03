import { Fragment, useEffect, useState } from "react";
import { useSkillsStore } from "../../store/useSkillsStore";
import { useAiTools } from "../../hooks/useAiTools";
import SkillCard from "../../components/skills/SkillCard";
import InstallDialog from "../../components/skills/InstallDialog";
import SearchInput from "../../components/ui/SearchInput";
import LoadingSpinner from "../../components/ui/LoadingSpinner";
import EmptyState from "../../components/ui/EmptyState";
import Badge from "../../components/ui/Badge";
import { Download } from "lucide-react";

export default function HubPage() {
  const { hubSkills, loading, error, fetchHubSkills } = useSkillsStore();
  const { tools } = useAiTools();
  const [search, setSearch] = useState("");
  const [installTarget, setInstallTarget] = useState<string | null>(null);

  useEffect(() => {
    fetchHubSkills();
  }, []);

  const filtered = hubSkills.filter(
    (s) =>
      s.name.toLowerCase().includes(search.toLowerCase()) ||
      s.description.toLowerCase().includes(search.toLowerCase())
  );

  const installSkill = installTarget
    ? hubSkills.find((s) => s.dir_name === installTarget)
    : null;

  return (
    <div className="p-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Skills Hub</h1>
          <p className="mt-1 text-sm text-gray-500">
            Shared skills available for installation across AI tools
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="info">{hubSkills.length} skills</Badge>
          <div className="w-64">
            <SearchInput
              value={search}
              onChange={setSearch}
              placeholder="Search hub skills..."
            />
          </div>
        </div>
      </div>

      {loading && <LoadingSpinner text="Loading hub skills..." />}
      {error && (
        <div className="mt-6 rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {error}
        </div>
      )}

      {!loading && filtered.length === 0 && (
        <div className="mt-8">
          <EmptyState
            title="No shared skills found"
            description="Skills hub is empty or ~/.agents/skills/ does not exist"
          />
        </div>
      )}

      <div className="mt-6 grid grid-cols-[1fr_auto] items-start gap-x-2 gap-y-3">
        {filtered.map((skill) => (
          <Fragment key={skill.dir_name}>
            <SkillCard skill={skill} basePath="/skills/hub" />
            <div className="self-start pt-3">
              <button
                onClick={() => setInstallTarget(skill.dir_name)}
                className="flex items-center gap-1 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-700"
              >
                <Download className="h-3 w-3" />
                Install
              </button>
            </div>
          </Fragment>
        ))}
      </div>

      {installSkill && installTarget && (
        <InstallDialog
          skillName={installTarget}
          tools={tools}
          installedIn={installSkill.installed_in}
          onClose={() => setInstallTarget(null)}
          onComplete={() => {
            fetchHubSkills();
            setInstallTarget(null);
          }}
        />
      )}
    </div>
  );
}
