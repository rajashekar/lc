# LLM Client Documentation

This directory contains the documentation website for LLM Client (lc), built with Docusaurus.

## Development

### Prerequisites

- Node.js 18 or higher
- npm or yarn

### Installation

```bash
cd docs-site
npm install
```

### Local Development

```bash
npm start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

### Build

```bash
npm run build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Deployment

The documentation is automatically deployed to Vercel when changes are pushed to the main branch.

### Manual Deployment

1. Install Vercel CLI:
   ```bash
   npm i -g vercel
   ```

2. Deploy:
   ```bash
   vercel --prod
   ```

## Structure

```
docs-site/
├── docs/                    # Markdown documentation files
│   ├── intro.md            # Homepage
│   ├── getting-started/    # Installation and quick start
│   ├── features/           # Core features
│   ├── advanced/           # Advanced features
│   ├── commands/           # Command reference
│   ├── providers/          # Provider-specific guides
│   └── development/        # Development guides
├── src/                    # React components and custom CSS
├── static/                 # Static assets
├── docusaurus.config.js    # Site configuration
├── sidebars.js            # Sidebar configuration
└── package.json           # Dependencies
```

## Adding Documentation

1. Create a new `.md` file in the appropriate directory under `docs/`
2. Add front matter:
   ```markdown
   ---
   id: unique-id
   title: Page Title
   sidebar_position: 1
   ---
   ```
3. Write your content in Markdown
4. Update `sidebars.js` if needed

## Configuration

- Site configuration: `docusaurus.config.js`
- Sidebar structure: `sidebars.js`
- Custom styling: `src/css/custom.css`

## Vercel Setup

To deploy to Vercel:

1. Create a new project on Vercel
2. Connect your GitHub repository
3. Set the root directory to `docs-site`
4. Add these environment variables to your GitHub repository:
   - `VERCEL_TOKEN`
   - `VERCEL_ORG_ID`
   - `VERCEL_PROJECT_ID`

The GitHub Action will automatically deploy on push to main.