## Weights and Fees Calculation

### Introduction

Transaction weights and fees ensure network efficiency, fairness, and security in Substrate-based blockchains. These concepts are designed to manage resource allocation and incentivize proper usage of the blockchain infrastructure.

### Transaction Weights

Transaction weight measures the computational resources required to process a transaction. It is determined by factors such as the complexity of the transaction logic and the amount of data involved. Transactions with higher weights consume more resources and thus contribute to the overall load on the network.

### Transaction Fees

Transaction fees in Substrate-based blockchains are crucial for efficiently managing network resources and sustaining economic viability. They regulate resource allocation by ensuring transactions consuming more computational resources incur higher fees, discouraging spam, and promoting fair use of network capacity.
The fees are computed as follows:
`fee = base_fee + weight2fee * fee_multiplier + length2fee + tip`

## Fees in Duniter

### Fees Implementation Details

Implementing a zero-fee chain in Duniter involves configuring the blockchain to waive transaction fees when the current block weight is below a specified target. This approach aims to promote accessibility and encourage participation by eliminating fees during periods of lower network activity.
However, transaction fees are applied when the block weight or length surpasses the defined targets to ensure network security and stability during increased usage. Additionally, leveraging the fee multiplier mechanism helps deter potential prolonged network attacks by dynamically adjusting fee levels based on previous network conditions.
Duniter members benefit from the quota system, which refunds transaction fees during high network activity periods.

Fees are computed as follows:
* If `current_weight < 0.25 * max_weight` and `current_length < 0.25 * max_length` and `fee_multiplier = 1`, ie. normal load:
`fee = 0`
* If `current_weight > 0.25 * max_weight` or `current_length > 0.25 * max_length` or `fee_multiplier > 1`, ie. heavy usage (approximately more than 135 transactions per second):
`fee = `5cĞ1 + extrinsic_weight * (5cĞ1/base_extrinsic_weight)* fee_multiplier + extrinsic_length/100 + tip`

The multiplier is updated as follows:
* If `current_weight > 0.25 * max_weight` or `current_length > 0.25 * max_length`:
`Min(fee_multiplier += 1, 10)`
* If `current_weight < 0.25 * max_weight` and `current_length < 0.25 * max_length`:
`Max(fee_multiplier -= 1, 1)`
