# Yellowstone Vixen Parser Samples

This document provides reference schemas and sample JSON responses for `SubscribeUpdate` messages from the Yellowstone Vixen gRPC streaming endpoint. These samples are based on the proto definitions in each parser's crate (e.g., `crates/[parser]/proto/[parser].proto`). Use these when integrating parsers to understand the structure of parsed data for accounts (`ProgramState`) and transactions (`ProgramIxs`).

## Overview

- **Endpoint**: `grpcurl -plaintext 127.0.0.1:3030 vixen.stream.ProgramStreams/Subscribe`
- **Request**: `{"program": "[ProgramPubkey]"}`
- **Response**: Streaming JSON of `SubscribeUpdate` messages.
- **Key Fields**:
  - `update_oneof`: Contains `account` (with `parsed_state`), `transaction` (with `parsed_ixs`), or `block_meta`.
  - Parsed data is embedded in `parsed_state` (for accounts) or `parsed_ixs` (for transactions).

Samples use fictional values but accurate structures. Refer to proto files for full field definitions.

## Meteora Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        InitializeLbPairIx initialize_lb_pair = 1;
        InitializePermissionLbPairIx initialize_permission_lb_pair = 2;
        InitializeCustomizablePermissionlessLbPairIx initialize_customizable_permissionless_lb_pair = 3;
        InitializeBinArrayBitmapExtensionIx initialize_bin_array_bitmap_extension = 4;
        InitializeBinArrayIx initialize_bin_array = 5;
        AddLiquidityIx add_liquidity = 6;
        AddLiquidityByWeightIx add_liquidity_by_weight = 7;
        AddLiquidityByStrategyIx add_liquidity_by_strategy = 8;
        AddLiquidityByStrategyOneSideIx add_liquidity_by_strategy_one_side = 9;
        AddLiquidityOneSideIx add_liquidity_one_side = 10;
        RemoveLiquidityIx remove_liquidity = 11;
        InitializePositionIx initialize_position = 12;
        InitializePositionPdaIx initialize_position_pda = 13;
        InitializePositionByOperatorIx initialize_position_by_operator = 14;
        UpdatePositionOperatorIx update_position_operator = 15;
        SwapIx swap = 16;
        SwapExactOutIx swap_exact_out = 17;
        SwapWithPriceImpactIx swap_with_price_impact = 18;
        WithdrawProtocolFeeIx withdraw_protocol_fee = 19;
        InitializeRewardIx initialize_reward = 20;
        FundRewardIx fund_reward = 21;
        UpdateRewardFunderIx update_reward_funder = 22;
        UpdateRewardDurationIx update_reward_duration = 23;
        ClaimRewardIx claim_reward = 24;
        ClaimFeeIx claim_fee = 25;
        ClosePositionIx close_position = 26;
        UpdateBaseFeeParametersIx update_base_fee_parameters = 27;
        UpdateDynamicFeeParametersIx update_dynamic_fee_parameters = 28;
        IncreaseOracleLengthIx increase_oracle_length = 29;
        InitializePresetParameterIx initialize_preset_parameter = 30;
        ClosePresetParameterIx close_preset_parameter = 31;
        ClosePresetParameter2Ix close_preset_parameter2 = 32;
        RemoveAllLiquidityIx remove_all_liquidity = 33;
        SetPairStatusIx set_pair_status = 34;
        MigratePositionIx migrate_position = 35;
        MigrateBinArrayIx migrate_bin_array = 36;
        UpdateFeesAndRewardsIx update_fees_and_rewards = 37;
        WithdrawIneligibleRewardIx withdraw_ineligible_reward = 38;
        SetActivationPointIx set_activation_point = 39;
        RemoveLiquidityByRangeIx remove_liquidity_by_range = 40;
        AddLiquidityOneSidePreciseIx add_liquidity_one_side_precise = 41;
        GoToABinIx go_to_a_bin = 42;
        SetPreActivationDurationIx set_pre_activation_duration = 43;
        SetPreActivationSwapAddressIx set_pre_activation_swap_address = 44;
        SetPairStatusPermissionlessIx set_pair_status_permissionless = 45;
        InitializeTokenBadgeIx initialize_token_badge = 46;
        CreateClaimProtocolFeeOperatorIx create_claim_protocol_fee_operator = 47;
        CloseClaimProtocolFeeOperatorIx close_claim_protocol_fee_operator = 48;
        InitializePresetParameter2Ix initialize_preset_parameter2 = 49;
        InitializeLbPair2Ix initialize_lb_pair2 = 50;
        InitializeCustomizablePermissionlessLbPair2Ix initialize_customizable_permissionless_lb_pair2 = 51;
        ClaimFee2Ix claim_fee2 = 52;
        ClaimReward2Ix claim_reward2 = 53;
        AddLiquidity2Ix add_liquidity2 = 54;
        AddLiquidityByStrategy2Ix add_liquidity_by_strategy2 = 55;
        AddLiquidityOneSidePrecise2Ix add_liquidity_one_side_precise2 = 56;
        RemoveLiquidity2Ix remove_liquidity2 = 57;
        RemoveLiquidityByRange2Ix remove_liquidity_by_range2 = 58;
        Swap2Ix swap2 = 59;
        SwapExactOut2Ix swap_exact_out2 = 60;
        SwapWithPriceImpact2Ix swap_with_price_impact2 = 61;
        ClosePosition2Ix close_position2 = 62;
        UpdateFeesAndReward2Ix update_fees_and_reward2 = 63;
        ClosePositionIfEmptyIx close_position_if_empty = 64;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        BinArrayBitmapExtension bin_array_bitmap_extension = 1;
        BinArray bin_array = 2;
        ClaimFeeOperator claim_fee_operator = 3;
        LbPair lb_pair = 4;
        Oracle oracle = 5;
        Position position = 6;
        PositionV2 position_v2 = 7;
        PresetParameter2 preset_parameter2 = 8;
        PresetParameter preset_parameter = 9;
        TokenBadge token_badge = 10;
    }
}
```

### Sample Account Update (LbPair)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "LbPairPubkey12345678901234567890123456789012",
        "owner": "MeteoraProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "lb_pair": {
          "active_id": 123,
          "bin_step_seed": 456,
          "base_factor": 789,
          "min_bin_id": 100,
          "max_bin_id": 200,
          "system_program": "SystemProgramId123456789012345678901234567890",
          "program_mint": "MintPubkey12345678901234567890123456789012",
          "base_x": 1000,
          "base_y": 2000
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (SwapIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "swap": {
            "accounts": {
              "lb_pair": "LbPairPubkey12345678901234567890123456789012",
              "user": "UserPubkey12345678901234567890123456789012",
              "bin_array_bitmap_extension": "BitmapExtPubkey12345678901234567890123456789012"
            },
            "data": {
              "amount_in": 1000000,
              "amount_out_min": 950000,
              "active_id": 123
            }
          }
        }
      ]
    }
  }
}
```

## Boop Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        BuyTokenIx buy_token = 1;
        SellTokenIx sell_token = 2;
        CreateTokenIx create_token = 3;
        WithdrawIx withdraw = 4;
        DepositIx deposit = 5;
        CreateAmmIx create_amm = 6;
        AddLiquidityIx add_liquidity = 7;
        RemoveLiquidityIx remove_liquidity = 8;
        SwapIx swap = 9;
        ClaimFeeIx claim_fee = 10;
        CreateLockEscrowIx create_lock_escrow = 11;
        LockIx lock = 12;
        ClaimIx claim = 13;
        ExtendLockIx extend_lock = 14;
        SplitIx split = 15;
        MergeIx merge = 16;
        TransferIx transfer = 17;
        CloseEscrowIx close_escrow = 18;
        CreateAmmV2Ix create_amm_v2 = 19;
        AddLiquidityV2Ix add_liquidity_v2 = 20;
        RemoveLiquidityV2Ix remove_liquidity_v2 = 21;
        SwapV2Ix swap_v2 = 22;
        ClaimFeeV2Ix claim_fee_v2 = 23;
        CloseAmmIx close_amm = 24;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        BondingCurve bonding_curve = 1;
        Amm amm = 2;
        LockEscrow lock_escrow = 3;
        AmmV2 amm_v2 = 4;
    }
}
```

### Sample Account Update (BondingCurve)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "BondingCurvePubkey12345678901234567890123456789012",
        "owner": "BoopProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "bonding_curve": {
          "virtual_token_reserves": 1000000000,
          "virtual_sol_reserves": 500000000,
          "real_token_reserves": 900000000,
          "real_sol_reserves": 450000000,
          "token_total_supply": 1000000000,
          "complete": false
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (BuyTokenIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "buy_token": {
            "accounts": {
              "user": "UserPubkey12345678901234567890123456789012",
              "mint": "MintPubkey12345678901234567890123456789012",
              "bonding_curve": "BondingCurvePubkey12345678901234567890123456789012"
            },
            "data": {
              "amount": 1000000,
              "maximum_sol_cost": 500000
            }
          }
        }
      ]
    }
  }
}
```

## Moonshot Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        TokenMintIx token_mint = 1;
        BuyIx buy = 2;
        SellIx sell = 3;
        MigrateFundsIx migrate_funds = 4;
        ConfigInitIx config_init = 5;
        ConfigUpdateIx config_update = 6;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        ConfigAccount config_account = 1;
        CurveAccount curve_account = 2;
    }
}
```

### Sample Account Update (CurveAccount)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "CurveAccountPubkey12345678901234567890123456789012",
        "owner": "MoonshotProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "curve_account": {
          "virtual_token_reserves": 1000000000,
          "virtual_sol_reserves": 500000000,
          "real_token_reserves": 900000000,
          "real_sol_reserves": 450000000,
          "token_total_supply": 1000000000,
          "complete": false
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (BuyIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "buy": {
            "accounts": {
              "user": "UserPubkey12345678901234567890123456789012",
              "mint": "MintPubkey12345678901234567890123456789012",
              "curve": "CurveAccountPubkey12345678901234567890123456789012"
            },
            "data": {
              "token_amount": 1000000,
              "collateral_amount": 500000,
              "fixed_side": 0,
              "slippage_bps": 100
            }
          }
        }
      ]
    }
  }
}
```

## Jupiter Swap Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        ClaimIx claim = 1;
        ClaimWithTokenLedgerIx claim_with_token_ledger = 2;
        ExactOutRouteIx exact_out_route = 3;
        RouteIx route = 4;
        RouteWithTokenLedgerIx route_with_token_ledger = 5;
        SetTokenLedgerIx set_token_ledger = 6;
        SharedAccountsExactOutRouteIx shared_accounts_exact_out_route = 7;
        SharedAccountsRouteIx shared_accounts_route = 8;
        SharedAccountsRouteWithTokenLedgerIx shared_accounts_route_with_token_ledger = 9;
        SharedAccountsExactOutRouteWithTokenLedgerIx shared_accounts_exact_out_route_with_token_ledger = 10;
        SharedAccountsRouteWithTokenLedgerIx shared_accounts_route_with_token_ledger = 11;
        SharedAccountsExactOutRouteWithTokenLedgerIx shared_accounts_exact_out_route_with_token_ledger = 12;
        SharedAccountsRouteWithTokenLedgerIx shared_accounts_route_with_token_ledger = 13;
        SharedAccountsRouteWithTokenLedgerIx shared_accounts_route_with_token_ledger = 14;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        TokenLedger token_ledger = 1;
    }
}
```

### Sample Account Update (TokenLedger)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "TokenLedgerPubkey12345678901234567890123456789012",
        "owner": "JupiterProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "token_ledger": {
          "token_account": "TokenAccountPubkey12345678901234567890123456789012",
          "amount": 1000000,
          "delegate": "DelegatePubkey12345678901234567890123456789012",
          "state": 1
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (RouteIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "route": {
            "accounts": {
              "token_program": "TokenProgramId123456789012345678901234567890",
              "user_transfer_authority": "AuthorityPubkey12345678901234567890123456789012",
              "user_source_token_account": "SourceTokenPubkey12345678901234567890123456789012",
              "user_destination_token_account": "DestTokenPubkey12345678901234567890123456789012",
              "destination_token_account": "DestTokenPubkey12345678901234567890123456789012",
              "destination_mint": "MintPubkey12345678901234567890123456789012",
              "platform_fee_account": "FeeAccountPubkey12345678901234567890123456789012",
              "event_authority": "EventAuthorityPubkey12345678901234567890123456789012",
              "program": "JupiterProgramId123456789012345678901234567890"
            },
            "data": {
              "amount": 1000000,
              "other_amount_threshold": 950000,
              "route_plan": [
                {
                  "swap": {
                    "meteora": {}
                  },
                  "percent": 100,
                  "input_index": 0,
                  "output_index": 1
                }
              ],
              "quoted_out_amount": 950000,
              "slippage_bps": 100,
              "platform_fee_bps": 10
            }
          }
        }
      ]
    }
  }
}
```

## Raydium Launchpad Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        BuyExactInIx buy_exact_in = 1;
        BuyExactOutIx buy_exact_out = 2;
        SellExactInIx sell_exact_in = 3;
        SellExactOutIx sell_exact_out = 4;
        CreatePoolIx create_pool = 5;
        InitializePoolIx initialize_pool = 6;
        UpdatePoolIx update_pool = 7;
        DepositIx deposit = 8;
        WithdrawIx withdraw = 9;
        SwapBaseInIx swap_base_in = 10;
        SwapBaseOutIx swap_base_out = 11;
        ConfigLpVaultIx config_lp_vault = 12;
        ConfigLpVaultV2Ix config_lp_vault_v2 = 13;
        WithdrawLpIx withdraw_lp = 14;
        WithdrawLpV2Ix withdraw_lp_v2 = 15;
        UpdatePlatformConfigIx update_platform_config = 16;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        GlobalConfig global_config = 1;
        PlatformConfig platform_config = 2;
        PoolState pool_state = 3;
        VestingRecord vesting_record = 4;
    }
}
```

### Sample Account Update (PoolState)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "PoolStatePubkey12345678901234567890123456789012",
        "owner": "RaydiumLaunchpadProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "pool_state": {
          "amm_config": "AmmConfigPubkey12345678901234567890123456789012",
          "amm_authority": "AuthorityPubkey12345678901234567890123456789012",
          "amm_open_orders": "OpenOrdersPubkey12345678901234567890123456789012",
          "amm_target_orders": "TargetOrdersPubkey12345678901234567890123456789012",
          "amm_coin_vault": "CoinVaultPubkey12345678901234567890123456789012",
          "amm_pc_vault": "PcVaultPubkey12345678901234567890123456789012",
          "withdraw_queue": "WithdrawQueuePubkey12345678901234567890123456789012",
          "amm_withdraw_queue": "AmmWithdrawQueuePubkey12345678901234567890123456789012",
          "amm_lp_vault": "LpVaultPubkey12345678901234567890123456789012",
          "serum_program_id": "SerumProgramId123456789012345678901234567890",
          "serum_market": "MarketPubkey12345678901234567890123456789012",
          "serum_coin_vault_account": "CoinVaultPubkey12345678901234567890123456789012",
          "serum_pc_vault_account": "PcVaultPubkey12345678901234567890123456789012",
          "serum_vault_signer": "VaultSignerPubkey12345678901234567890123456789012",
          "coin_mint_address": "CoinMintPubkey12345678901234567890123456789012",
          "pc_mint_address": "PcMintPubkey12345678901234567890123456789012",
          "lp_mint_address": "LpMintPubkey12345678901234567890123456789012",
          "pool_coin_token_account": "PoolCoinTokenPubkey12345678901234567890123456789012",
          "pool_pc_token_account": "PoolPcTokenPubkey12345678901234567890123456789012",
          "pool_withdraw_queue": "PoolWithdrawQueuePubkey12345678901234567890123456789012",
          "pool_temp_lp_token_account": "TempLpTokenPubkey12345678901234567890123456789012",
          "amm_owner": "OwnerPubkey12345678901234567890123456789012",
          "pool_lp_token_account": "PoolLpTokenPubkey12345678901234567890123456789012"
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (BuyExactInIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "buy_exact_in": {
            "accounts": {
              "user": "UserPubkey12345678901234567890123456789012",
              "pool_state": "PoolStatePubkey12345678901234567890123456789012",
              "input_token_account": "InputTokenPubkey12345678901234567890123456789012",
              "output_token_account": "OutputTokenPubkey12345678901234567890123456789012",
              "input_vault": "InputVaultPubkey12345678901234567890123456789012",
              "output_vault": "OutputVaultPubkey12345678901234567890123456789012",
              "last_observation": "ObservationPubkey12345678901234567890123456789012"
            },
            "data": {
              "amount_in": 1000000,
              "minimum_amount_out": 950000
            }
          }
        }
      ]
    }
  }
}
```

## Pump Swaps Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        BuyIx buy = 1;
        SellIx sell = 2;
        CreateIx create = 3;
        CompleteIx complete = 4;
        SetParamsIx set_params = 5;
        AddLiquidityIx add_liquidity = 6;
        RemoveLiquidityIx remove_liquidity = 7;
        SwapIx swap = 8;
        ClaimFeeIx claim_fee = 9;
        CreateTokenIx create_token = 10;
        BuyCreatorIx buy_creator = 11;
        WithdrawIx withdraw = 12;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        BondingCurve bonding_curve = 1;
        GlobalConfig global_config = 2;
        Pool pool = 3;
    }
}
```

### Sample Account Update (BondingCurve)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "BondingCurvePubkey12345678901234567890123456789012",
        "owner": "PumpSwapsProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "bonding_curve": {
          "virtual_token_reserves": 1000000000,
          "virtual_sol_reserves": 500000000,
          "real_token_reserves": 900000000,
          "real_sol_reserves": 450000000,
          "token_total_supply": 1000000000,
          "complete": false
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (BuyIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "buy": {
            "accounts": {
              "user": "UserPubkey12345678901234567890123456789012",
              "mint": "MintPubkey12345678901234567890123456789012",
              "bonding_curve": "BondingCurvePubkey12345678901234567890123456789012",
              "associated_bonding_curve": "AssocBondingCurvePubkey12345678901234567890123456789012",
              "associated_user": "AssocUserPubkey12345678901234567890123456789012",
              "system_program": "SystemProgramId123456789012345678901234567890",
              "token_program": "TokenProgramId123456789012345678901234567890",
              "rent": "RentSysvarPubkey12345678901234567890123456789012"
            },
            "data": {
              "amount": 1000000,
              "max_sol_cost": 500000
            }
          }
        }
      ]
    }
  }
}
```

## Pumpfun Parser

### Schema (ProgramIxs)
```proto3
message ProgramIxs {
    oneof ix_oneof {
        BuyIx buy = 1;
        SellIx sell = 2;
        CreateIx create = 3;
        CompleteEventIx complete_event = 4;
        SetParamsIx set_params = 5;
        AddLiquidityIx add_liquidity = 6;
        RemoveLiquidityIx remove_liquidity = 7;
        SwapIx swap = 8;
        ClaimFeeIx claim_fee = 9;
        CreateTokenIx create_token = 10;
        UpdateGlobalAuthorityIx update_global_authority = 11;
    }
}
```

### Schema (ProgramState)
```proto3
message ProgramState {
    oneof state_oneof {
        BondingCurve bonding_curve = 1;
        Global global = 2;
    }
}
```

### Sample Account Update (BondingCurve)
```json
{
  "update_oneof": {
    "account": {
      "slot": 123456789,
      "account": {
        "pubkey": "BondingCurvePubkey12345678901234567890123456789012",
        "owner": "PumpfunProgramId123456789012345678901234567890",
        "lamports": 1000000,
        "data": "base64_encoded_data",
        "executable": false,
        "rent_epoch": 0
      },
      "parsed_state": {
        "bonding_curve": {
          "virtual_token_reserves": 1000000000,
          "virtual_sol_reserves": 500000000,
          "real_token_reserves": 900000000,
          "real_sol_reserves": 450000000,
          "token_total_supply": 1000000000,
          "complete": false
        }
      },
      "is_startup": false
    }
  }
}
```

### Sample Transaction Update (BuyIx)
```json
{
  "update_oneof": {
    "transaction": {
      "slot": 123456790,
      "signature": "base64_sig_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "is_vote": false,
      "account_keys": ["Pubkey1", "Pubkey2", "Pubkey3"],
      "recent_blockhash": "Blockhash12345678901234567890123456789012",
      "instructions": [
        {
          "program_id_index": 0,
          "accounts": [1, 2],
          "data": "base64_ix_data"
        }
      ],
      "meta": {
        "err": null,
        "fee": 5000,
        "pre_balances": [1000000, 2000000],
        "post_balances": [950000, 2050000]
      },
      "parsed_ixs": [
        {
          "buy": {
            "accounts": {
              "user": "UserPubkey12345678901234567890123456789012",
              "mint": "MintPubkey12345678901234567890123456789012",
              "bonding_curve": "BondingCurvePubkey12345678901234567890123456789012",
              "associated_bonding_curve": "AssocBondingCurvePubkey12345678901234567890123456789012",
              "associated_user": "AssocUserPubkey12345678901234567890123456789012",
              "system_program": "SystemProgramId123456789012345678901234567890",
              "token_program": "TokenProgramId123456789012345678901234567890",
              "rent": "RentSysvarPubkey12345678901234567890123456789012"
            },
            "data": {
              "amount": 1000000,
              "max_sol_cost": 500000
            }
          }
        }
      ]
    }
  }
}
```

## Block Meta Update (Shared)

### Sample
```json
{
  "update_oneof": {
    "block_meta": {
      "slot": 123456789,
      "blockhash": "Blockhash12345678901234567890123456789012",
      "rewards": [
        {
          "pubkey": "ValidatorPubkey12345678901234567890123456789012",
          "lamports": 1000000,
          "post_balance": 500000000,
          "reward_type": "Fee",
          "commission": 10
        }
      ],
      "block_time": 1693526400,
      "block_height": 123456789,
      "executed_transaction_count": 1000,
      "entries_count": 2000
    }
  }
}
```

For full proto definitions, see the respective `crates/[parser]/proto/[parser].proto` files.
