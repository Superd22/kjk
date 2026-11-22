# Hosted GitHub Actions runners with a free open-source offer and macOS arm64

Research date: 2026-11-19. Every claim below cites the vendor page that owns it. Pages were read through a fetch tool that summarises them, so exact prices should be re-checked before anyone commits money.

## Short answer

Yes, but only barely. Two third-party services combine a free offer with macOS arm64 runners. Neither one is a clear win over GitHub's own free runners for a public repository.

1. GitHub itself is the baseline and the best default. Standard GitHub-hosted runners are free and unlimited on public repositories, and they include Apple silicon macOS images (`macos-15`, `macos-26`, `macos-latest`). The label `macos-14` was retired on 2 November 2026. The catch is concurrency, because a free-plan account can run at most 5 macOS jobs at the same time.
2. Blacksmith is the only third-party vendor with both a monthly free allowance that covers macOS and an open-source sponsorship program. The free allowance is small for macOS. One minute on a 6 vCPU macOS runner counts as 20 of the 3,000 free minutes, so the allowance gives about 150 macOS minutes per month. The open-source program is by nomination, and its terms are not published.
3. MacStadium has a free program for open-source projects that gives hosted Apple silicon Mac minis. The application page currently says the program is full and applications are closed. It is also not a hosted GitHub Actions runner service, so the project would have to run its own self-hosted runner on the Mac.
4. WarpBuild, Namespace, Depot and Ubicloud have no open-source program on the pages I could read. Cirrus Runners no longer accepts new customers. BuildJet has shut down. RunsOn does not offer macOS. Actuated does not offer macOS.

## Comparison table

| Vendor | Free tier or open-source offer | macOS arm64 | macOS sizes | Open-source access needs an application | Main catch |
|---|---|---|---|---|---|
| GitHub-hosted | Free and unlimited standard runners on public repos | Yes, `macos-15`, `macos-26`, `macos-latest`, `xcode-27` | 3 vCPU (M1), 7 GB RAM, 14 GB SSD | No | 5 concurrent macOS jobs on the Free plan, and macOS jobs can queue at busy times |
| Blacksmith | 3,000 free minutes per month per organization, plus an open-source sponsorship program | Yes, M4 | 6 vCPU 24 GB, 12 vCPU 48 GB | Yes, by nomination, no criteria published | A macOS minute costs 20 free minutes, so the free allowance is about 150 macOS minutes |
| WarpBuild | Free sign-up with a credit limit, no open-source program found | Yes, M4 Pro and M5 Pro | 6 vCPU 14 GB, 12 vCPU 28 GB | Not applicable | Paid at $0.08 and $0.16 per minute, and free credit amount is not published |
| Namespace | Developer plan at $0 pay as you go, 30-day trial, no open-source program found | Yes, M4 Pro or M5 Max | Up to 12 vCPU and 56 GB | Not applicable | macOS minutes use a 10x multiplier, no free minutes stated |
| Depot | 7-day trial, no permanent free tier, no open-source program found | Yes, M4 and M2 | 8 vCPU, 24 GB | Not applicable | Starts at $20 per month, and macOS capacity can queue |
| Ubicloud | 1,250 free minutes per month, no open-source discount | No macOS found | Not applicable | Not applicable | Linux only |
| Cirrus Runners | Not open to new customers | Was yes | Not applicable | Not applicable | Cirrus Labs joined OpenAI, and Cirrus CI shut down on 1 June 2026 |
| BuildJet | None | No | Not applicable | Not applicable | Shut down |
| RunsOn | Free license for open-source projects, but compute is paid to AWS | No | Not applicable | Not stated | macOS is not supported because of Apple's 24-hour minimum on EC2 Mac |
| Actuated | None found | No, Linux only | Not applicable | Not applicable | Flat $150 per month per server |
| MacStadium | Free hosted Mac mini for open-source projects | Yes, Apple silicon Mac mini | Not stated | Yes, waitlist | Applications are closed, and you run your own runner |

## GitHub-hosted runners (baseline)

- The standard-runner table for public repositories lists the arm64 labels `macos-latest`, `macos-15`, `macos-26` and `xcode-27`, each with 3 CPUs (M1), 7 GB RAM and 14 GB SSD. It also lists Intel labels `macos-15-intel` and `macos-26-intel` with 4 CPUs and 14 GB RAM. The same page states: "Use of the standard GitHub-hosted runners is free and unlimited on public repositories." Source: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- The label `macos-14` is not on that page. GitHub's changelog says: "The macOS 14 runner image will be retired on November 2, 2026." It covers `macos-14`, `macos-14-large` and `macos-14-xlarge`. Source: https://github.blog/changelog/2026-10-01-github-actions-macos-14-runner-image-retirement
- Concurrency limits for standard runners are 20 total jobs and 5 macOS jobs on the Free plan, 40 and 5 on Pro, 60 and 5 on Team, and 500 and 50 on Enterprise. The macOS limit is shared with larger runners. The page also says macOS jobs "may take longer to start" during high demand, and that GitHub Support can raise the limits. Source: https://docs.github.com/en/actions/reference/limits
- I did not verify from a primary page whether the public-repo concurrency limit differs from the Free plan row. Treat 5 as the working number.

## Blacksmith

- Pricing page: 3,000 free minutes per month on Ubuntu x64, Ubuntu ARM, Windows x64 and macOS M4 runners. The macOS M4 runner costs $0.08 per minute. Source: https://www.blacksmith.sh/pricing
- The same page describes an open-source program. It lists Celery, Ladybird, Zen and Limbo as current projects. It says eligibility needs an actively maintained public repo, a permissive license and clear community usage, and that applications go through a form on the page. Source: https://www.blacksmith.sh/pricing
- Blacksmith's Celery story says the sponsorship gives away its compute and storage to open-source projects, and that you nominate a project through the link on the pricing page. It publishes no criteria, steps or deadlines. Source: https://www.blacksmith.sh/customer-stories/celery
- Runner documentation: the labels are `blacksmith-6vcpu-macos-latest` and `blacksmith-12vcpu-macos-latest`, plus pinned `macos-15`, `macos-26` and `macos-27`. The sizes are 6 vCPU with 24 GB and 150 GB storage, and 12 vCPU with 48 GB and 250 GB storage, on Apple M4. Each organization gets 3,000 x64 2 vCPU minutes per month, and one macOS 6 vCPU minute counts as 20 of them. A 10-minute job therefore uses 200 minutes. Source: https://docs.blacksmith.sh/blacksmith-runners/overview
- Catch one is the 20x multiplier, which leaves about 150 macOS minutes per month on the standard free allowance. Catch two is that the open-source sponsorship may grant more, but its limits are not published, so a project would have to ask.
- Blacksmith's own pricing docs page (https://docs.blacksmith.sh/blacksmith-runners/pricing) returned a 404 when I tried it.

## WarpBuild

- Pricing page: macOS arm64 costs $0.08 per minute for 6 vCPU and 14 GB, and $0.16 per minute for 12 vCPU and 28 GB, on M4 Pro and M5 Pro hosts. Sign-up needs no payment method until "the free usage or credit limits" are reached, but the page does not say how large they are. It lists no open-source program. Source: https://www.warpbuild.com/pricing
- Runner documentation: the tags are `warp-macos-26-arm64-6x` and `-12x`, `warp-macos-15-arm64-6x` and `-12x`, and `warp-macos-14-arm64-6x`. Storage is 120 GB and 270 GB. The page mentions no open-source program or free macOS credit. Source: https://www.warpbuild.com/docs/ci/cloud-runners
- A web search found no WarpBuild open-source program. The only free item found was a promo credit of $50 on a partner page, which is not an open-source offer.

## Namespace

- Pricing page: the Developer plan costs $0 and is pay as you go, with a 30-day free trial and no credit card required. The compute multiplier is 10 for macOS. The Team plan costs $100 per month with 100,000 unit minutes. The page lists no open-source program. Source: https://namespace.so/pricing
- macOS page: instances go up to 12 vCPU and 56 GB of RAM on Apple M4 Pro or M5 Max. Source: https://namespace.so/compute/macos
- GitHub Actions documentation: macOS runners are arm64 and are chosen through a profile label such as `namespace-profile-big-apple`. Specific macOS shapes are not listed on that page. Source: https://namespace.so/docs/solutions/github-actions
- No free macOS minutes are stated on any of these pages.

## Depot

- Pricing page: the Developer plan costs $20 per month and includes 2,000 GitHub Actions minutes, then $0.006 per minute for the base Linux runner. There is a 7-day trial and no permanent free tier. macOS is included on all plans. The page lists an FAQ question about open-source discounts, but the answer was not in the text I could read, so I cannot say whether a program exists. Source: https://depot.dev/pricing
- macOS runners: `depot-macos-26` (M4), `depot-macos-15` and `depot-macos-14` (M2), each with 8 CPUs, 24 GB and 400 GB disk at $0.08 per minute. The page warns that macOS capacity is not fully elastic and can queue. Source: https://depot.dev/docs/github-actions/runner-types
- The runner overview page says nothing about open source. Source: https://depot.dev/docs/github-actions/overview
- A person should ask Depot directly before ruling out an open-source discount.

## Ubicloud

- The GitHub Actions page advertises 1,250 free minutes per month, x64 and arm64 Linux runners, and says the platform itself is open source. It mentions no macOS runners and no open-source program. Source: https://ubicloud.com/use-cases/github-actions

## Cirrus Runners

- Cirrus Labs announced it is joining OpenAI. The announcement says Cirrus Runners no longer accepts new customers, existing customers are supported through their current contract periods, and "Cirrus CI will shut down effective Monday, June 1, 2026." Tart, Vetu and Orchard will move to a more permissive license, and licensing fees have stopped. Source: https://cirruslabs.org/
- Cirrus Runners was previously the best-known macOS arm64 option for open source on GitHub Actions, but it is closed to new users.

## BuildJet

- The BuildJet page carries the notice "BuildJet for GitHub Actions has been shut down." It lists no free tier and no macOS runners. Source: https://buildjet.com/for-github-actions

## RunsOn

- Pricing page: the Free license (0 EUR) covers open-source, non-profit and personal non-commercial use, in exchange for a public link to runs-on.com. The customer still pays AWS for compute. Source: https://runs-on.com/pricing/
- macOS page: RunsOn does not support macOS runners because of the 24-hour minimum allocation period on EC2 Mac instances. It recommends using GitHub-hosted runners or a dedicated macOS provider for macOS jobs. Source: https://runs-on.com/runners/macos/

## Actuated

- Pricing page: the Self-Service plan is $150 per month for the first server and $125 for each additional one, with unlimited minutes. The page says "Linux x86_64 & Arm supported" and mentions no macOS and no open-source offer. Source: https://actuated.com/pricing

## MacStadium and Orka

- FOSS application page: it says "Applications Closed" and "The program is currently full." Visitors can leave contact details to be told when spots open. The page lists no criteria and no hardware, and does not mention GitHub Actions. Source: https://www.macstadium.com/foss-application
- Pricing page: individually hosted Mac minis run from $109 per month (M2, 8 GB) to $349 per month (M4 Pro, 48 GB), billed monthly and prepaid with no free trial. Orka pricing goes through sales. The page does not mention GitHub Actions runners. Source: https://macstadium.com/pricing
- MacStadium's own blog post about supporting open source describes the program's earlier terms. I did not fetch it, so its details (no commercial funding, at least three months of development) come from a search summary and should be confirmed at https://macstadium.com/blog/macstadium-supports-the-open-source-community

## Gaps in this research

- I could not read the Namespace and Depot open-source answers, if any exist. Both need a direct question.
- Blacksmith's open-source terms are not public beyond the nomination link.
- Tenki (https://www.tenki.cloud) came up as another runner vendor, but its pricing page redirected elsewhere and I did not read it, so it is not assessed.
