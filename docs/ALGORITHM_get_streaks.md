# `get_streaks` Algorithm

This document outlines the algorithm used to identify "Focused Streaks" and "Distractive Streaks" from user activity data. This is used to analyze user behavior, calculate focus streaks, and provide insights into productivity patterns.

## Layman's Explanation

Imagine you want to know when you were truly "in the zone" versus when you were getting distracted. This algorithm helps figure that out by looking at your app usage.

1.  **Scoring Apps:** First, we give each app a "productivity score". You can tag apps as "Productive" (e.g., coding tools, document editors) or "Distractive" (e.g., games, social media). Productive apps get a high score, and distractive ones get a low score.
2.  **Finding Distractions:** We look for times you used distractive apps for more than a few seconds. If you quickly switch between a few different distracting apps, we bundle those moments together into a single "Distractive Streak".
3.  **Finding Focus:** Next, we look for times you used productive apps. For these to count as "Focused Streaks," two important rules apply:
    *   You must be using them for a meaningful amount of time.
    *   This usage must *not* happen during a Distractive Streak. A quick check of a distractive app can "break" your focus.
4.  **Putting It Together:** The final output is a timeline showing all your focused and distractive streaks, giving you a clear picture of your day.

## Assumptions

1.  **App Scores**: Each app can be assigned a score. A higher score means more productive, a lower score means more distractive. Apps without an explicit score are considered neutral (score of 0).
2.  **Mutually Exclusive Categories**: The threshold for a "focus" app is greater than or equal to the threshold for a "distractive" app. This means an app cannot be both a focus app and a distractive app simultaneously.
    \[
    \theta_{F,min\_score} \ge \theta_{D,max\_score}
    \]
3.  **Continuous Usage**: The `usages` table contains records of continuous, non-overlapping blocks of time spent in an application.

## Rigorous Explanation

The algorithm identifies and merges qualifying app usages into maximal "Focused Streaks" and "Distractive Streaks" over a given time range \([T_{start}, T_{end}]\).

### Definitions

1.  **App Sets**:
    *   Let \(A\) be the set of all applications.
    *   The score of an app \(a \in A\) is denoted by \(score(a)\).
    *   **Focus Apps** (\(A_F\)): The set of apps considered productive.
        \[ A_F = \{a \in A \mid score(a) > \theta_{F,min\_score}\} \]
    *   **Distractive Apps** (\(A_D\)): The set of apps considered distractive.
        \[ A_D = \{a \in A \mid score(a) < \theta_{D,max\_score}\} \]

2.  **Initial Usage Sets**:
    *   Let \(U\) be the set of all `usages` within the query time range \([T_{start}, T_{end}]\). Each usage \(u \in U\) is a tuple \((a, t_{start}, t_{end})\) where \(a \in A\).
    *   **Initial Distractive Usages** (\(U'_D\)): Usages of distractive apps that meet a minimum duration requirement.
        \[ U'_D = \{u \in U \mid u.a \in A_D \land (u.t_{end} - u.t_{start}) \ge \theta_{D,min\_dur}\} \]
    *   **Initial Focus Usages** (\(U'_F\)): Usages of focus apps that meet a minimum duration requirement.
        \[ U'_F = \{u \in U \mid u.a \in A_F \land (u.t_{end} - u.t_{start}) \ge \theta_{F,min\_dur}\} \]

### Per-App Usage Merging (Pre-Filtering)

Real-world data often splits a continuous interaction with the **same** application into multiple short `usage` rows (for instance, because the window briefly lost focus).  Relying on the raw rows would cause many short fragments to be discarded by the minimum-duration filter.

To address this, **before** we apply the duration threshold we first **merge adjacent usages that belong to the same application** if the gap between them is not greater than the corresponding “max gap” setting.  Formally, for each app \(a\) we build a sequence of its usages ordered by their start time and merge runs as follows:

• Two consecutive usages \(u_i, u_{i+1}\) where \(u_i.a = u_{i+1}.a\) are merged when
\[
    u_{i+1}.t_{start} - u_i.t_{end} \le \theta_{D,max\_gap} \quad (\text{distractive side})
\]
or
\[
    u_{i+1}.t_{start} - u_i.t_{end} \le \theta_{F,max\_gap} \quad (\text{focus side}).
\]

After the transitive closure of this rule, we obtain **per-app merged usages**
\(\widetilde{U}_D\) and \(\widetilde{U}_F\).  The minimum-duration thresholds are then evaluated **on these merged intervals**:

\[
    U'_D = \{u \in \widetilde{U}_D \mid (u.t_{end} - u.t_{start}) \ge \theta_{D,min\_dur}\},\qquad
    U'_F = \{u \in \widetilde{U}_F \mid (u.t_{end} - u.t_{start}) \ge \theta_{F,min\_dur}\}.
\]

The remainder of the algorithm (calculating DSs and FSs, pruning focus by distraction, and joining across applications) operates on these filtered, merged sets.

### Algorithm Steps

The algorithm proceeds in three main stages:

#### 1. Calculate Distractive Streaks (DSs)

First, we calculate the maximal streaks of distraction.

1.  **Filter**: Select the set of Initial Distractive Usages, \(U'_D\).
2.  **Merge**: Merge adjacent usages from \(U'_D\). Two usages \(u_i, u_j \in U'_D\) (with \(u_i.t_{end} < u_j.t_{start}\)) are considered adjacent if the gap between them is sufficiently small:
    \[ (u_j.t_{start} - u_i.t_{end}) \le \theta_{D,max\_gap} \]
    This merging is performed transitively until no more merges can be made. The result is a set of maximal, non-overlapping **Distractive Streaks** \({DS_1, DS_2, \dots}\).

#### 2. Calculate Focused Streaks (FSs)

Next, we calculate the maximal streaks of focus, ensuring they are not tainted by distractions.

1.  **Filter**: Select the set of Initial Focus Usages, \(U'_F\).
2.  **Prune**: Remove any focus usage from \(U'_F\) that intersects with any calculated Distractive Streak. A usage \(u \in U'_F\) is removed if:
    \[ \exists DS_k \text{ such that } [u.t_{start}, u.t_{end}] \cap [DS_k.t_{start}, DS_k.t_{end}] \ne \emptyset \]
    Let the resulting set be \(U''_F\).
3.  **Merge**: Merge adjacent usages from \(U''_F\). Two usages \(u_i, u_j \in U''_F\) (with \(u_i.t_{end} < u_j.t_{start}\)) are merged if two conditions are met:
    a. The gap between them is sufficiently small:
    \[ (u_j.t_{start} - u_i.t_{end}) \le \theta_{F,max\_gap} \]
    b. The gap itself does not overlap with any Distractive Streak. This prevents merging two focus blocks that are separated by a distraction.
    \[ \forall DS_k, [u_i.t_{end}, u_j.t_{start}] \cap [DS_k.t_{start}, DS_k.t_{end}] = \emptyset \]
    This merging is also performed transitively. The result is a set of maximal, non-overlapping **Focused Streaks** \({FS_1, FS_2, \dots}\).

#### 3. Final Output

The final result is the union of the two sets of streaks, \(\{DS_k\}\) and \(\{FS_j\}\), sorted by their start time. Each streak is labeled as either "distractive" or "focus". 