# Deploying LLM Client Documentation to Vercel

This guide will help you deploy the LLM Client documentation to Vercel and set up automatic deployments.

## Prerequisites

- A GitHub account with the lc repository
- A Vercel account (free tier is sufficient)
- Node.js 18+ installed locally

## Step 1: Initial Setup

1. **Install dependencies locally** (optional, for testing):
   ```bash
   cd docs-site
   npm install
   npm run build
   ```

2. **Test locally** (optional):
   ```bash
   npm start
   ```
   Visit http://localhost:3000 to preview

## Step 2: Import to Vercel

1. Go to [vercel.com](https://vercel.com) and sign in
2. Click "Add New..." → "Project"
3. Import your GitHub repository
4. Configure the project:
   - **Framework Preset**: Other
   - **Root Directory**: `docs-site`
   - **Build Command**: `npm run build`
   - **Output Directory**: `build`
   - **Install Command**: `npm install`

5. Click "Deploy"

## Step 3: Configure Custom Domain

1. In your Vercel project, go to "Settings" → "Domains"
2. Add `lc.viwq.dev`
3. You'll see DNS records to add to your domain

### DNS Configuration

Add these records to your domain's DNS (at viwq.dev):

**For subdomain (recommended):**
```
Type: CNAME
Name: lc
Value: cname.vercel-dns.com
```

**OR for apex domain:**
```
Type: A
Name: @
Value: 76.76.21.21
```

4. Wait for DNS propagation (usually 5-30 minutes)
5. Vercel will automatically provision SSL certificates

## Step 4: Set Up GitHub Actions (Optional)

For more control over deployments, use GitHub Actions:

1. **Get Vercel tokens**:
   - Install Vercel CLI: `npm i -g vercel`
   - Run `vercel login`
   - Run `vercel link` in the `docs-site` directory
   - Get your tokens from `.vercel/project.json`

2. **Add secrets to GitHub**:
   - Go to your repository → Settings → Secrets → Actions
   - Add these secrets:
     - `VERCEL_TOKEN`: Your Vercel token (from `vercel whoami`)
     - `VERCEL_ORG_ID`: From `.vercel/project.json`
     - `VERCEL_PROJECT_ID`: From `.vercel/project.json`

3. **The workflow is already set up** in `.github/workflows/deploy-docs.yml`

## Step 5: Verify Deployment

1. Push changes to the `main` branch
2. Check GitHub Actions for build status
3. Visit https://lc.viwq.dev to see your documentation

## Updating Documentation

### Automatic Updates

Any changes to files in `docs-site/` pushed to `main` will trigger automatic deployment.

### Manual Updates

1. Make changes to markdown files in `docs-site/docs/`
2. Test locally: `npm start`
3. Commit and push to GitHub
4. Deployment happens automatically

## Environment Variables (if needed)

In Vercel project settings → Environment Variables:

- `ALGOLIA_APP_ID` - For search functionality (optional)
- `ALGOLIA_API_KEY` - For search functionality (optional)

## Troubleshooting

### Build Fails

1. Check build logs in Vercel dashboard
2. Ensure Node.js version is 18+
3. Try clearing cache: Vercel → Settings → Functions → Clear Cache

### Domain Not Working

1. Verify DNS records are correct
2. Check SSL certificate status in Vercel
3. Wait for full DNS propagation (up to 48 hours)

### 404 Errors

1. Ensure `docs-site` is set as root directory
2. Check that `docusaurus.config.js` has correct `baseUrl`
3. Verify build output includes all pages

## Monitoring

- **Analytics**: Enable in Vercel dashboard
- **Performance**: Check Web Vitals in Vercel
- **Errors**: Set up error notifications in project settings

## Next Steps

1. Customize the theme in `src/css/custom.css`
2. Add Algolia DocSearch for better search functionality
3. Set up preview deployments for pull requests
4. Add more documentation content

## Support

- Vercel Documentation: https://vercel.com/docs
- Docusaurus Documentation: https://docusaurus.io/docs
- GitHub Actions: https://docs.github.com/en/actions