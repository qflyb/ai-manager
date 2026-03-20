import { useNavigate } from "react-router-dom";
import { BrainCircuit, Puzzle, Plug, FileText } from "lucide-react";

const features = [
  {
    id: "skills",
    name: "AI Skills",
    description: "Manage skills across all AI coding tools",
    icon: BrainCircuit,
    path: "/skills",
    color: "bg-indigo-50 text-indigo-600",
    ready: true,
  },
  {
    id: "plugins",
    name: "Plugins",
    description: "Manage Claude Code compatible plugins from local or GitHub",
    icon: Puzzle,
    path: "/plugins",
    color: "bg-violet-50 text-violet-600",
    ready: true,
  },
  {
    id: "mcp",
    name: "MCP Servers",
    description: "Manage Model Context Protocol servers",
    icon: Plug,
    path: "/mcp",
    color: "bg-emerald-50 text-emerald-600",
    ready: false,
  },
  {
    id: "rules",
    name: "AI Rules",
    description: "Manage rules and instructions for AI tools",
    icon: FileText,
    path: "/rules",
    color: "bg-amber-50 text-amber-600",
    ready: false,
  },
];

export default function Home() {
  const navigate = useNavigate();

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold text-gray-900">AI Manager</h1>
      <p className="mt-1 text-sm text-gray-500">
        Manage your AI coding tools from one place
      </p>

      <div className="mt-8 grid grid-cols-3 gap-5">
        {features.map((f) => (
          <button
            key={f.id}
            onClick={() => f.ready && navigate(f.path)}
            disabled={!f.ready}
            className={`group relative flex flex-col items-start rounded-xl border border-gray-200 bg-white p-6 text-left transition-shadow ${
              f.ready
                ? "cursor-pointer hover:shadow-md"
                : "cursor-not-allowed opacity-60"
            }`}
          >
            <div className={`rounded-lg p-2.5 ${f.color}`}>
              <f.icon className="h-6 w-6" />
            </div>
            <h2 className="mt-4 text-base font-semibold text-gray-900">
              {f.name}
            </h2>
            <p className="mt-1 text-sm text-gray-500">{f.description}</p>
            {!f.ready && (
              <span className="mt-3 inline-block rounded-full bg-gray-100 px-2.5 py-0.5 text-xs font-medium text-gray-500">
                Coming soon
              </span>
            )}
          </button>
        ))}
      </div>
    </div>
  );
}
