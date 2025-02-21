// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightOpenAPI, { openAPISidebarGroups } from "starlight-openapi";

// https://astro.build/config
export default defineConfig({
    base: "/docs",
    integrations: [
        starlight({
            title: "Kromer Docs",
            social: {
                github: "https://github.com/ReconnectedCC/kromer",
            },
            plugins: [
                // Generate the OpenAPI documentation pages.
                starlightOpenAPI([
                    {
                        base: "api/kromer",
                        label: "Kromer",
                        schema: "./open_api.yaml",
                    },
                ]),
            ],
            sidebar: [
                {
                    label: "Guides",
                    items: [
                        // Each item here is one entry in the navigation menu.
                        { label: "Example Guide", slug: "guides/example" },
                    ],
                },
                {
                    label: "Reference",
                    autogenerate: { directory: "reference" },
                },
                ...openAPISidebarGroups,
            ],
        }),
    ],
});
