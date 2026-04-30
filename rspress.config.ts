import { defineConfig } from "@rspress/core";
import { pluginShiki } from "@rspress/plugin-shiki";

export default defineConfig({
  title: "Agent Gateway",
  description: "A unified gateway for managing multiple AI coding tools",
  themeConfig: {
    logo: "https://agent-gateway.ai/favicon.ico",
    nav: [
      {
        text: "指南",
        link: "/guide/01-intro",
        activeMatch: "/guide/",
      },
      {
        text: "模块",
        link: "/core",
        activeMatch: "/(core|cli|api|gui|core-ts|node-ts)",
      },
      {
        text: "设计",
        link: "/design",
        activeMatch: "/(design|process|dev)",
      },
    ],
    sidebar: {
      "/guide/": [
        {
          text: "指南",
          items: [
            { text: "简介", link: "/guide/01-intro" },
            { text: "安装", link: "/guide/02-install" },
            { text: "套餐", link: "/guide/03-plan" },
            { text: "CLI", link: "/guide/04-cli" },
            { text: "降级", link: "/guide/05-fallback" },
            { text: "配额", link: "/guide/06-quota" },
            { text: "API", link: "/guide/07-api" },
            { text: "协议转换", link: "/guide/08-converter" },
            { text: "GUI", link: "/guide/09-gui" },
            { text: "插件", link: "/guide/10-plugin" },
            { text: "故障排除", link: "/guide/11-troubleshooting" },
          ],
        },
      ],
      "/": [
        {
          text: "模块文档",
          items: [
            { text: "agw-core", link: "/core" },
            { text: "agw-cli", link: "/cli" },
            { text: "agw-api", link: "/api" },
            { text: "agw-gui", link: "/gui" },
            { text: "@agent-gateway/core", link: "/core-ts" },
            { text: "@agent-gateway/node", link: "/node-ts" },
          ],
        },
        {
          text: "设计文档",
          items: [
            { text: "架构设计", link: "/design" },
            { text: "开发流程", link: "/process" },
            { text: "开发指南", link: "/dev" },
          ],
        },
      ],
    },
    socialLinks: [
      {
        icon: "github",
        mode: "link",
        content: "https://github.com/veaba/agent-gateway",
      },
    ],
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2024-present Agent Gateway",
    },
  },
  plugins: [pluginShiki()],
  docSources: [],
});
