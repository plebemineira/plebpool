# Hashrate estimation

Share hashes can be interpreted as unsigned 256 bit integers (`U256`).

A target is established as a hash value, which when interpreted as a `U256` acts as threshold. Valid shares must have a hash whose integer interpretation is smaller than the target.

We establish $T = \{ T_1, T_2, ..., T_{} \}$ as the set of targets where:
- $T_1$ = `"0x0000000000000000000000000000000000000000000000000000000000000001"`
- $T_2$ = `"0x0000000000000000000000000000000000000000000000000000000000000003"`
- $T_3$ = `"0x0000000000000000000000000000000000000000000000000000000000000007"`
- $T_4$ = `"0x000000000000000000000000000000000000000000000000000000000000000F"`
- $T_5$ = `"0x000000000000000000000000000000000000000000000000000000000000001F"`
- $...$
- $T_{max}$ = `"0x00000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"`

$T_{max}$ can also be interpreted as:

$U256(T_{max}) = 26959946667150639794667015087019630673637144422540572481103610249215$

For some given target $T_x$, we calculate its difficulty $D_x$ as:

$$D_x = \frac{U256(T_{max})}{U256(T_x)}$$

Let's define:
- The mining epoch $e$ as the time window between two blocks being found on the network
- $M_e = \{ M_1^e, M_2^e, ..., M_N^e \}$ as the set of miners working on the pool during epoch $e$
- $h^e = \{ h_1^e, h_2^e, ..., h_N^e \}$ as the set of miner hashrates during epoch $e$ (the unit here is `Hashes/s`).
- $\hat{h}_x^e = \{ \hat{h}_1^e, \hat{h}_2^e, ..., \hat{h}_N^e \}$ as the set of estimated miner hashrates during epoch $e$
- $d$ as the difficulty of each share
- $D_x$ as the difficulty threshold of the communication channel between pool and miner $M_x^e$ during epoch $e$
- $S_x^e$ as the set of valid shares submitted by miner $M_x^e$, under difficulty target $D_x$, during epoch $e$

In order to optimize bandwidth, the pool opens communication channels with lower difficulty threshold for small miners, while larger miners get channels with higher difficulty thresholds.
These thresholds are adjusted dynamically via SV2/SV1 messages. If a miner repeatedly submits shares that violate the established thresholds, the pool drops the connection as a mechanism to avoid spam.

A share is considered valid if $d \geq D_x$. The probability of a valid share is given by [1]:
$$P(d \geq D_x) = \frac{2^{16}-1}{2^{48}D_x} \approx \frac{1}{2^{32}D_x}$$

If the pool collects shares from miner $M_x^e$ for a time window $t_e$ (in seconds), then the total number of valid observed shares $|S_x^e|$ is:

$$|S_x^e| = \frac{h_x^et_e}{2^{32}D_x}$$

so we can estimate hashrate $\hat{h}_x^e$ as:

$$ \hat{h}^e_x = \frac{|S_x^e|2^{32}D_x}{t_e}$$

if $t_e$ is sufficiently large, then the expectation $E[\hat{h}_x^e] \to E[h_x^e]$, and we can assume $\hat{h}_x^e$ is a reliable estimation of the hashrate $h_x^e$.

This set of estimated hashrates $\hat{h}_x^e$ (relative to all active miners during epoch $e$) is used as input for [reward distribution](reward-distribution.md).

## References

1: https://arxiv.org/abs/1112.4980
