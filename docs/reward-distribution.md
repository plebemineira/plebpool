# Reward distribution

Reward distribution is similar to Pay-Per-Last-N-Shares (PPLNS).

The mining pool finds a block with $\theta_B^e$ sats in the coinbase (subsidy + fees).

The entire reward of $\theta_B^e$ on-chain sats is used to open a LN channel while paying $F_{LN}$ in fees.

$$ \hat{\theta_C^e} = \theta_C^e - F_{LN} $$

Based on the set of [estimated hashrates](hashrate-estimation.md) $\hat{h}^e$, each miner $M_x^e$ is rewarded $R_x^e$ sats under the following formula:

$$ R_x^e = \frac{\hat{h}_x^e}{\sum \hat{h}^e_i } * \hat{\theta_C^e} = \frac{\frac{|S_x^e|2^{32}D_x}{t_e}}{\sum \frac{|S_i^e|2^{32}D_i}{t_e}} * \hat{\theta_C^e} = \frac{|S_x^e|D_x}{\sum |S_i^e|D_i} * \hat{\theta_C^e} $$

For example, let's imagine 3 miners were active during epoch $e$.
- Alice submitted the set $S_A^e$ of shares with difficulty above $D_A$
- Bob submitted the set $S_B^e$ of shares with difficulty above $D_B$
- Carol submitted the set $S_C^e$ of shares with difficulty above $D_C$

The reward distribution is:
- $R_A^e = \frac{\hat{h}_A^e}{\hat{h}^e_A + \hat{h}^e_B + \hat{h}^e_C} * \hat{\theta_C^e} = \frac{|S_A^e|D_A}{|S_A^e|D_A + |S_B^e| D_B + |S_C^e| D_C} * \hat{\theta_C^e}$ [ `sats` ]
- $R_B^e = \frac{\hat{h}_B^e}{\hat{h}^e_A + \hat{h}^e_B + \hat{h}^e_C} * \hat{\theta_C^e} =  \frac{|S_B^e|D_B}{|S_A^e|D_A + |S_B^e| D_B + |S_C^e| D_C} * \hat{\theta_C^e}$ [ `sats` ]
- $R_C^e = \frac{\hat{h}_C^e}{\hat{h}^e_A + \hat{h}^e_B + \hat{h}^e_C} * \hat{\theta_C^e} = \frac{|S_C^e|D_C}{|S_A^e|D_A + |S_B^e| D_B + |S_C^e| D_C} * \hat{\theta_C^e}$ [ `sats` ]
- $R_A^e + R_B^e + R_C^e = \hat{\theta_C^e}$


## Pricing shares

If the pool finds a block during epoch $e$, it opens a LN channel with the entire rewards of the coinbase while paying $F_{LN}$ fees:

$$ \hat{\theta_C^e} = \theta_C^e - F_{LN} $$

So there's a total $\hat{\theta_C^e}$ sats available to be claimed via LN, and miner $M_x^e$ has the right to claim a reward $R_x^e$ according to:

$$ R_x^e = \frac{|S_x^e|D_x}{\sum_{i}{} |S_i^e|D_i} * \hat{\theta_C^e} $$

where:
- $S_x^e$ is the set of shares (valid under $D_x$) submitted by miner $M_x^e$ on the last mining epoch $e$.
- $D_x$ is the difficulty threshold for the connection with miner $M_x^e$, calculated as $D_x = \frac{U256(T_{max})}{U256(T_x)}$
- $S_i^e$ is the set of all shares submitted to the pool under difficulty $D_i$ on the last: mining epoch $e$.

Therefore, each share is priced as:

$$ \beta^e (s_{x,j}^e) = \frac{R_x^e}{|S_x^e|} = \frac{D_x}{\sum |S_i^e|D_i} * \theta_C^e$$

In case a block is found on the network (by someone else, not the pool), then the pool starts a new mining epoch $e+1$ while discarding shares from db.

## Economic limitations

This reward distribution strategy puts the variance risk on the side of the miner.

## References

1: https://arxiv.org/abs/1112.4980