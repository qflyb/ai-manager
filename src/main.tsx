import React from "react";
import ReactDOM from "react-dom/client";
import { createHashRouter, RouterProvider } from "react-router-dom";
import App from "./App";
import Home from "./pages/Home";
import SkillsDashboard from "./pages/skills/SkillsDashboard";
import ToolDetailPage from "./pages/skills/ToolDetailPage";
import SkillDetailPage from "./pages/skills/SkillDetailPage";
import HubPage from "./pages/skills/HubPage";
import BySkillPage from "./pages/skills/BySkillPage";
import PluginsPage from "./pages/plugins/PluginsPage";
import PluginDetailPage from "./pages/plugins/PluginDetailPage";
import Settings from "./pages/Settings";
import CacheManagement from "./pages/CacheManagement";
import "./App.css";

if (import.meta.env.DEV) {
  import("react-grab");
}

const router = createHashRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <Home /> },
      { path: "skills", element: <SkillsDashboard /> },
      { path: "skills/tools/:toolId", element: <ToolDetailPage /> },
      {
        path: "skills/tools/:toolId/:skillName",
        element: <SkillDetailPage />,
      },
      { path: "skills/by-skill", element: <BySkillPage /> },
      { path: "skills/hub", element: <HubPage /> },
      { path: "skills/hub/:skillName", element: <SkillDetailPage /> },
      { path: "plugins", element: <PluginsPage /> },
      { path: "plugins/:pluginId", element: <PluginDetailPage /> },
      { path: "settings", element: <Settings /> },
      { path: "settings/cache", element: <CacheManagement /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
