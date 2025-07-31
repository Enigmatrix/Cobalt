# `get_streaks` Algorithm

This document outlines the algorithm used to identify "Focused Streaks" and "Distractive Streaks" from user activity data. This is used to analyze user behavior, calculate focus streaks, and provide insights into productivity patterns.

## Layman's Explanation

Imagine you want to know when you were truly "in the zone" versus when you were getting distracted. This algorithm helps figure that out by looking at your app usage.

1. **Continuous Usages** Continuously using (same app / different apps of the same type - either focus or distractive) for a minimum duration is considered a "streak".

1. **Streak Gaps** Streaks can be merged if the gap between them is not greater than the configured maximum gap.

1. **Distraction topples Focus** If a distractive streak intersects with a focus streak, the focus streak is cut short. Similarly, focus streaks cannot merge across a distractive streak.

1. **Final Output** The final output is a timeline showing all your focused and distractive streaks, giving you a clear picture of your day.

### Breakdown

1.  **Scoring Apps:** First, we take each app's "productivity score". You can tag apps as "Productive" (e.g., coding tools, document editors) or "Distractive" (e.g., games, social media). Productive apps get a high score, and distractive ones get a low score (e.g. negative).

1. **Initial Usage Blocks** Adjacent usages of distractive apps (no time gap between them) are merged into a single distractive streak, as long as the streak is at least the configured minimum duration. This is to avoid counting short, spurious usage blocks as distractive streaks.

1. **Distractive Streaks** We merge adjacent distractive streaks into a single distractive streak, as long as the gap between them is not greater than the configured maximum gap.

1. **Initial Focus Streaks** Adjacent usages of focus apps (no time gap between them) that are not interrupted by a distractive streak are merged into a single focus streak, as long as the streak is at least the configured minimum duration. This is to avoid counting short, spurious usage blocks as focus streaks and to avoid counting focus streaks that are interrupted by a distractive streak.

1. **Focus Streaks** We merge adjacent focus streaks into a single focus streak, as long as the gap between them is not greater than the configured maximum gap and there is no distractive streak in between.

## Assumptions

1.  **App Scores** Each app can be assigned a score. A higher score means more productive, a lower score means more distractive. Apps without an explicit score are considered neutral (score of 0).
2.  **Mutually Exclusive Categories** The threshold for a "focus" app is greater than or equal to the threshold for a "distractive" app. This means an app cannot be both a focus app and a distractive app simultaneously.
    ```math
    \theta_{F,min\_score} \ge \theta_{D,max\_score}
    ```
3.  **Continuous Usage** The `usages` table contains records of continuous, non-overlapping blocks of time spent in an application (except for system events like screen off, screen on, etc.).

## Rigorous Explanation

Given a query window $[T_{start}, T_{end}]$, the goal is to derive two collections of non-overlapping intervals

```math
\mathcal{DS} = \lbrace DS_1,DS_2,\dots \rbrace, \qquad
\mathcal{FS} = \lbrace FS_1,FS_2,\dots \rbrace
```

where each $DS_k$ represents continuous distraction and each $FS_j$ represents continuous focus.

### Parameters

| Symbol | Meaning |
|---|---|
| $\theta_{F,min\_score}$ | Minimum score for an app to be considered *focus* |
| $\theta_{D,max\_score}$ | Maximum score for an app to be considered *distractive* |
| $\theta_{F,min\_dur}$ | Minimum duration of a focus usage |
| $\theta_{D,min\_dur}$ | Minimum duration of a distractive usage |
| $\theta_{F,max\_gap}$ | Maximum gap allowed when merging focus streaks |
| $\theta_{D,max\_gap}$ | Maximum gap allowed when merging distractive streaks |

### 1. Categorising Applications

Let $A$ be the set of all applications and $score: A \to \mathbb{R}$ the productivity-score function.

```math
A_F = \lbrace a\in A \mid score(a) > \theta_{F,min\_score} \rbrace, \qquad
A_D = \lbrace a\in A \mid score(a) < \theta_{D,max\_score} \rbrace.
```

### 2. Raw Usages

All `usages` rows that intersect the window form

```math
U = \lbrace (a,t_{start},t_{end}) \mid t_{start} < T_{end},\; t_{end} > T_{start} \rbrace.
```

Note that $U$ is an ordered set sorted by start time.

### 3. Initial Distractive Streaks

Let

```math
U_D = \lbrace u \in U \mid u.a \in A_D \rbrace.
```

Apply zero–gap merging within $U_D$ and keep only intervals whose duration meets the minimum requirement:

```math
\widetilde{U}_D = \text{MergeGap}_{\text{dur}(g) = 0}(U_D),\qquad
\text{InitDS} = \lbrace u\in\widetilde{U}_D \mid \text{dur}(u) \ge \theta_{D,min\_dur} \rbrace.
```

where $\text{dur}(u) = t_{end} - t_{start}$ and

```math
S = \lbrace u_1,\dots,u_n \rbrace\text{ sorted by }\text{start}(u_i).
```

```math
\pi(1)=1,\quad
\pi(i)=\begin{cases}
\pi(i-1) & \text{if }\text{cond}(g = (\text{end}(u_{i-1}),\text{start}(u_{i})))\\
\pi(i-1)+1 & \text{otherwise}
\end{cases}\;(i\ge2).
```

```math
G_k = \lbrace u_i\mid \pi(i)=k \rbrace,\qquad k=1,\dots,\pi(n).
```

```math
\text{MergeGap}_{\text{cond}}(S)=\Bigl\lbrace \bigl(\min_{u\in G_k}\text{start}(u),\;\max_{u\in G_k}\text{end}(u)\bigr) : k=1,\dots,\pi(n) \Bigr\rbrace.
```

### 4. Distractive Streaks

Combine neighbouring distractive intervals when their separating gap is at most $\theta_{D,max\_gap}$ (transitively):

```math
\mathcal{DS} = \text{MergeGap}_{\text{dur}(g) \le \theta_{D,max\_gap} }(\text{InitDS}).
```

### 5. Initial Focus Streaks

Start with focus usages and remove anything that touches distraction:

```math
U_F = \lbrace u \in U \mid u.a \in A_F \rbrace, \qquad
U_F^{\text{clean}} = \lbrace u\in U_F \mid \forall DS\in\mathcal{DS}: u\cap DS = \varnothing \rbrace.
```

Produce contiguous focus runs and filter by duration:

```math
\widetilde{U}_F = \text{MergeGap}_{\text{dur}(g) = 0}(U_F^{\text{clean}}),\qquad
\text{InitFS} = \lbrace u\in\widetilde{U}_F \mid \text{dur}(u) \ge \theta_{F,min\_dur} \rbrace.
```

### 6. Focus Streaks

Merge adjacent focus intervals when (i) the gap $g = (\text{end}(u_i),\text{start}(u_{i+1}))$ is at most $\theta_{F,max\_gap}$ **and** (ii) the open gap is distraction-free (i.e. $\forall DS \in \mathcal{DS}: g \cap DS = \varnothing$):

```math
\mathcal{FS} = \text{MergeGap}_{\left( \text{dur}(g) \le \theta_{F,max\_gap} \right) \land \left( \forall DS \in \mathcal{DS}: g \cap DS = \varnothing \right)}(\text{InitFS}).
```

### 7. Clipping and Union

Each streak is clipped to the query window and labelled $is\_focused\in\{0,1\}$ (0 = distraction, 1 = focus).  The final timeline is the start-time-ordered union $\mathcal{DS}\cup\mathcal{FS}$.