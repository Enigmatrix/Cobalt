# `get_periods` Algorithm

This document outlines the algorithm used to identify "Focused Periods" and "Distractive Periods" from user activity data. This is used to analyze user behavior, calculate focus streaks, and provide insights into productivity patterns.

## Layman's Explanation

Imagine you want to know when you were truly "in the zone" versus when you were getting distracted. This algorithm helps figure that out by looking at your app usage.

1.  **Scoring Apps:** First, we give each app a "productivity score". You can tag apps as "Productive" (e.g., coding tools, document editors) or "Distractive" (e.g., games, social media). Productive apps get a high score, and distractive ones get a low score.
2.  **Finding Distractions:** We look for times you used distractive apps for more than a few seconds. If you quickly switch between a few different distracting apps, we bundle those moments together into a single "Distractive Period".
3.  **Finding Focus:** Next, we look for times you used productive apps. For these to count as "Focused Periods," two important rules apply:
    *   You must be using them for a meaningful amount of time.
    *   This usage must *not* happen during a Distractive Period. A quick check of a distractive app can "break" your focus.
4.  **Putting It Together:** The final output is a timeline showing all your focused and distractive periods, giving you a clear picture of your day.

## Assumptions

1.  **App Scores**: Each app can be assigned an integer score. A higher score means more productive, a lower score means more distractive. Apps without an explicit score are considered neutral (score of 0).
2.  **Mutually Exclusive Categories**: The threshold for a "focus" app is greater than or equal to the threshold for a "distractive" app. This means an app cannot be both a focus app and a distractive app simultaneously.
    \[
    \theta_{F,min\_score} \ge \theta_{D,max\_score}
    \]
3.  **Continuous Usage**: The `usages` table contains records of continuous, non-overlapping blocks of time spent in an application.

## Rigorous Explanation

The algorithm identifies and merges qualifying app usages into maximal "Focused Periods" and "Distractive Periods" over a given time range \([T_{start}, T_{end}]\).

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

### Algorithm Steps

The algorithm proceeds in three main stages:

#### 1. Calculate Distractive Periods (DPs)

First, we calculate the maximal periods of distraction.

1.  **Filter**: Select the set of Initial Distractive Usages, \(U'_D\).
2.  **Merge**: Merge adjacent usages from \(U'_D\). Two usages \(u_i, u_j \in U'_D\) (with \(u_i.t_{end} < u_j.t_{start}\)) are considered adjacent if the gap between them is sufficiently small:
    \[ (u_j.t_{start} - u_i.t_{end}) \le \theta_{D,max\_gap} \]
    This merging is performed transitively until no more merges can be made. The result is a set of maximal, non-overlapping **Distractive Periods** \({DP_1, DP_2, \dots}\).

#### 2. Calculate Focused Periods (FPs)

Next, we calculate the maximal periods of focus, ensuring they are not tainted by distractions.

1.  **Filter**: Select the set of Initial Focus Usages, \(U'_F\).
2.  **Prune**: Remove any focus usage from \(U'_F\) that intersects with any calculated Distractive Period. A usage \(u \in U'_F\) is removed if:
    \[ \exists DP_k \text{ such that } [u.t_{start}, u.t_{end}] \cap [DP_k.t_{start}, DP_k.t_{end}] \ne \emptyset \]
    Let the resulting set be \(U''_F\).
3.  **Merge**: Merge adjacent usages from \(U''_F\). Two usages \(u_i, u_j \in U''_F\) (with \(u_i.t_{end} < u_j.t_{start}\)) are merged if two conditions are met:
    a. The gap between them is sufficiently small:
    \[ (u_j.t_{start} - u_i.t_{end}) \le \theta_{F,max\_gap} \]
    b. The gap itself does not overlap with any Distractive Period. This prevents merging two focus blocks that are separated by a distraction.
    \[ \forall DP_k, [u_i.t_{end}, u_j.t_{start}] \cap [DP_k.t_{start}, DP_k.t_{end}] = \emptyset \]
    This merging is also performed transitively. The result is a set of maximal, non-overlapping **Focused Periods** \({FP_1, FP_2, \dots}\).

#### 3. Final Output

The final result is the union of the two sets of periods, \(\{DP_k\}\) and \(\{FP_j\}\), sorted by their start time. Each period is labeled as either "distractive" or "focus". 