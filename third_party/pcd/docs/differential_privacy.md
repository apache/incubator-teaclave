# Differential Privacy Manual

Unlike $k$-anonymity, DP is a property of a randomized mechanism/algorihtm that produces a dataset. The definition is controlled by a privacy budget $\varepsilon$: let $\cal X$ be the dataset domain and $x,x'$ be two neighboring dataset such that they differ at most one row. We say a randomized mechanism $\cal M$ is $\varepsilon$-differentially private if for all possible output $S$, the following inequality holds.

$$
\frac{\Pr[\mathcal{M}(x) = S]}{\Pr[\mathcal{M}(x) =S]} \leq e^\varepsilon.
$$

The intuition behind this definition is quite straightforward: to prevent from identifying single row in the dataset, we just need to ensure that *its effect on the entire dataset is negligible to the extent of a parameter $\varepsilon$.* Put it in a mathematical way, we use probability to describe such an intuition. There also exists a loose version that introduces another parameter called $\delta$, denoting the probability of failure:

$$
\frac{\Pr[\mathcal{M}(x) = S]}{\Pr[\mathcal{M}(x) =S]} \leq e^\varepsilon + \delta.
$$

> Bad thing is, we do not know how to select $\varepsilon$, yet. Common practice implies that this parameter is preferred to be below 1 and not above 10 at all.
> 

DP enjoys the following lovely properties.

1. Sequential composition: if $\cal M_1, M_2$ are $\varepsilon_1$- and $\varepsilon_2$-DP, then $\cal M = (M_1, M_2)$ is ($\varepsilon_1 + \varepsilon_2$)-DP.
2. Parallel composition: if $\cal M$ is $\varepsilon$-DP, and if we split a dataset $x$ into $k$ disjoint chunks such that $\bigcup_{i \in [k]} x_i = x$, then $\cal M(x_1),\cdots,M(x_k)$ is $\varepsilon$-DP.
3. Post processing: if $\cal$ $\cal M$ is $\varepsilon$-DP, then for any algorithm $\cal M'$, $\cal M'(M(x))$ is $\varepsilon$-DP. That is, there is no way to downgrade the privacy level of a DP-released dataset.
4. $k$-fold adaptive composition (e.g., loops): consider a set of mechanism $m_1,\cdots,m_k$, where each mechanism is $\varepsilon$-DP, and the input of the current mechanism is the output of the previous one. Then for any $\delta \ge 0$, the entire composition enjoys ($\varepsilon',\delta'$)-DP (or $\varepsilon', k\delta + \delta'$ if mechanisms are approximate DP) where
    
    $$
    \varepsilon' = 2\varepsilon \sqrt{2k\log(\frac{1}{\delta'})}.
    $$

## OpenDP

The Policy-Carrying Data framework utilizes the `OpenDP` programming framework for achieving differential privacy in which some terms need to be clarified.

* Measurement `opendp::measurement::*`: the randomized DP mechanism. E.g., Laplace machanism, Gaussian mechanism, etc.
* Transformation `opendp::transformation::*`: dataset transformation. E.g., clamping: upper- and lower-bound the original dataset as some queries have unbounded sesitivity which cannot be directly applied with DP mechanisms. See also PINQ.
* Composition: we can combine two DP mechanisms together and also track the privacy budget loss.
