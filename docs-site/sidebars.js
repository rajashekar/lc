/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  // By default, Docusaurus generates a sidebar from the docs folder structure
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/installation',
        'getting-started/quick-start',
      ],
    },
    {
      type: 'category',
      label: 'Commands',
      items: [
        'commands/overview',
        'commands/completions',
        'commands/config',
        'commands/keys',
        'commands/providers',
        'commands/models',
        'commands/chat',
        'commands/audio',
        'commands/embed',
        'commands/vectors',
        'commands/similar',
        'commands/search',
        'commands/templates',
        'commands/alias',
        'commands/sync',
        'commands/logs',
        'commands/proxy',
        'commands/web-chat-proxy',
        'commands/mcp',
        'commands/usage',
      ],
    },
    {
      type: 'category',
      label: 'Advanced Features',
      items: [
        'advanced/vector-database',
        'advanced/embeddings',
        'advanced/rag',
        'advanced/provider-management',
        'advanced/sync',
        'advanced/mcp',
        'advanced/vision'
      ],
    },
    {
      type: 'category',
      label: 'Library Usage',
      items: [
        'library-usage/overview',
        'library-usage/installation',
        'library-usage/basic-usage',
        'library-usage/configuration',
        'library-usage/advanced-features'
      ]
    },
    {
      type: 'category',
      label: 'Providers',
      items: [
        'providers/openai',
        'providers/anthropic',
        'providers/gemini',
        'providers/vertex-ai',
        'providers/vertex-ai-llama'
      ]
    },
    'troubleshooting',
    'faq',
  ],
};

export default sidebars;