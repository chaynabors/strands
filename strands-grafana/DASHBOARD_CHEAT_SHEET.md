# Strands Grafana Dashboards Cheat Sheet

## Dashboard 1: Strands Health
**Purpose:** High-level view of project health, velocity, and community engagement.

### Key Metrics Row (Top KPIs)
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **Open PRs (Current)** | Number of PRs waiting to be reviewed/merged right now | High numbers = review bottleneck. Shows current workload. |
| **Avg Merge Time (14d)** | Average hours from when a PR is opened to when it's merged | Lower is better. Shows how fast you're shipping code. |
| **Community PR % (30d)** | What percentage of merged PRs came from outside contributors | Higher = healthy open source project with engaged community |
| **Active Contributors (7d)** | Unique people who submitted PRs this week | Shows engagement momentum. Declining = warning sign. |
| **Merged (7d)** | How many PRs were merged this week | Shows shipping velocity. Are you moving fast? |
| **Stale PRs (7d+)** | PRs with no activity for 7+ days | These need attention! Contributors may abandon them. |

### Volume Section
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **PRs Merged (Daily)** | PRs shipped per day | Your delivery rate |
| **Issues Opened (Daily)** | New issues per day | Spike = possible bug or release issue |
| **Issues Closed (Daily)** | Issues resolved per day | Should roughly match opened to avoid backlog growth |
| **Total Open PRs (Current)** | Trend of open PR count over time | Rising = backlog growing faster than you're clearing it |

### Speed Section
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **Cycle Time (14-Day Rolling)** | Hours from PR open to merge, split by Team vs Community | Shows if community PRs are getting stuck vs team PRs |
| **Time to First Review (14-Day Rolling)** | Hours until first code review | Long waits frustrate contributors and slow shipping |
| **Time to First Response** | Hours until someone comments on a new issue/PR | Community expects quick acknowledgment (<24h ideal) |
| **Slow Reviews (>48h Wait)** | PRs waiting 2+ days for review | These contributors may get frustrated and leave |

### Quality Section
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **CI Failure Rate (7-Day Rolling)** | % of builds that fail | High = flaky tests or unstable main branch |
| **PR Acceptance Rate (30-Day Rolling)** | % of opened PRs that get merged | Low rate = contributors wasting effort on rejected PRs |
| **Code Churn (7-Day Rolling)** | Lines added+deleted per day | Unusually high = major refactoring or possible instability |
| **Backlog Growth (WoW)** | Week-over-week change in open PRs | Positive = falling behind. Negative = catching up. |

### Community Section
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **Total Stars (Current)** | GitHub stars over time | Growth indicator - shows project popularity/awareness |
| **Weekly Active Community Contributors** | Unique external contributors per week | Core health metric for open source |
| **Community PR Share (30-Day Rolling)** | % of work coming from community | Shows if project is truly community-driven |
| **Rejected PRs (Daily)** | PRs closed without merging | High = unclear contribution guidelines or low quality submissions |

### Contributor Insights Section
| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **New vs Returning Contributors** | First-timers vs repeat contributors | Healthy projects convert new → returning |
| **Contributor Retention Rate** | % of contributors who came back | Low retention = bad experience or one-time drive-by fixes |
| **Top Community Contributors** | Leaderboard of most active external contributors | Identify potential maintainers/champions |
| **Unique Contributors by Repository** | Which repos attract the most contributors | Shows where community interest is focused |
| **Cumulative Community Contributors** | Running total of all-time unique contributors | Overall community growth trajectory |

---

## Dashboard 2: Strands Team
**Purpose:** Internal team performance and workload balance.

| Metric | What It Shows | Why It Matters |
|--------|---------------|----------------|
| **📊 Team Scorecard** | Table showing each team member's PRs authored, reviews given, and overall score | Identifies who's contributing what. Score = PRs×2 + Reviews |
| **🏅 Category Leaders (30d)** | "Git Gladiator" (most PRs) and "Relentless Reviewer" (most reviews) | Fun recognition + identifies top performers |
| **📈 Team Review Balance (Weekly)** | Chart showing if team is reviewing more or less than authoring | Green (positive) = team keeping up with reviews. Red = falling behind. |
| **PR Source Breakdown** | Pie chart: Team vs Community vs Dependabot | Shows where merged work is coming from |
| **🔍 Reviews per Team Member** | Bar chart of review counts | Ensures review work is distributed fairly |

**Understanding the Scorecard columns:**
- **PRs Authored** — How many PRs this person merged
- **Reviews Given** — How many PRs this person reviewed
- **Balance** — Reviews minus PRs (positive = reviewing more, negative = writing more)
- **Score** — Overall contribution (PRs count double since they're harder)

---

## Dashboard 3: Strands Triage
**Purpose:** Action-oriented view for daily/weekly triage. What needs attention NOW?

| Panel | What It Shows | Action To Take |
|-------|---------------|----------------|
| **Team PRs** | Open PRs from team members with status & days open | Review approved ones. Ping stale ones. |
| **Community PRs** | Open PRs from external contributors | Prioritize these! First-timers flagged with 🆕 |
| **Untriaged Issues (No Milestone)** | Issues without a milestone assigned | Assign milestones or close if invalid |
| **Top Upvoted Issues** | Most 👍'd open issues | Shows what community wants most |
| **Most Discussed Issues** | Issues with most comments | May need decisions or be contentious |
| **⚠️ Stale PRs (7d+ No Activity)** | PRs going cold | Ping authors or close if abandoned |
| **🌟 First-Time Contributors** | PRs from people who've never had a PR merged | Give extra attention! Good experience = repeat contributor |
| **🤖 Dependabot PRs** | Automated dependency update PRs | Batch merge or review security updates |

---

## Quick Tips for Your Presentation

1. **⏱️ icon** = Panel follows the date picker (change dates to see different periods)
2. **"Current" or "Point-in-time"** = Shows right now, ignores date picker
3. **"Rolling"** = Smoothed average to reduce noise (e.g., 7-day or 14-day)
4. Use the **repo dropdown** to filter to specific repositories

**Key Story Points:**
- Health dashboard → "Here's how the project is doing overall"
- Team dashboard → "Here's how the team is performing"
- Triage dashboard → "Here's what needs attention this week"
