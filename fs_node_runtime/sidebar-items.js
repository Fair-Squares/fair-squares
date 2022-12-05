window.SIDEBAR_ITEMS = {"constant":[["EXISTENTIAL_DEPOSIT","Existential deposit."],["VERSION",""],["WASM_BINARY",""],["WASM_BINARY_BLOATY",""],["WEIGHT_PER_SECOND",""]],"enum":[["BalancesCall","Contains one variant per dispatchable that can be called by an extrinsic."],["Call",""],["DispatchClass","A generalized group of dispatch types."],["Event",""],["OriginCaller",""],["SystemCall","Contains one variant per dispatchable that can be called by an extrinsic."],["TimestampCall","Contains one variant per dispatchable that can be called by an extrinsic."]],"fn":[["native_version","The version information used to identify this runtime when compiled natively."]],"macro":[["construct_runtime","Construct a runtime, with the given name and the given pallets."],["parameter_types","Create new implementations of the `Get` trait."]],"mod":[["api",""],["constants","A set of constant values used in substrate runtime."],["opaque","Opaque types. These are used by the CLI to instantiate machinery that don’t need to know the specifics of the runtime. They can then be made to be agnostic over specific formats of data like extrinsics, allowing for them to continue syncing the network through upgrades to even the core data structures."]],"struct":[["ApprovalDeposit",""],["AsEnsureOriginWithArg",""],["AssetDeposit",""],["AssetsFees",""],["AttributeDepositBase",""],["BasicDeposit",""],["BlockExecutionWeight","Time to execute an empty block. Calculated by multiplying the Average with `1` and adding `0`."],["BlockHashCount",""],["Burn",""],["CheckDelay",""],["CheckPeriod",""],["CollectionDeposit",""],["ConstU128","Const getter for a basic type."],["ConstU32","Const getter for a basic type."],["ConstU64","Const getter for a basic type."],["ConstU8","Const getter for a basic type."],["CooloffPeriod",""],["CouncilMaxMembers",""],["CouncilMaxProposals",""],["CouncilMotionDuration",""],["DataDepositPerByte",""],["Delay",""],["DepositPerByte",""],["DontAllowCollectiveAndDemocracy",""],["EitherOfDiverse","“OR gate” implementation of `EnsureOrigin` allowing for different `Success` types for `L` and `R`, with them combined using an `Either` type."],["EnactmentPeriod",""],["EqualPrivilegeOnly","Implementation of [`PrivilegeCmp`] that only checks for equal origins."],["ExtrinsicBaseWeight","Time to execute a NO-OP extrinsic, for example `System::remark`. Calculated by multiplying the Average with `1` and adding `0`."],["FastTrackVotingPeriod",""],["FeesAccount",""],["FieldDeposit",""],["FundThreshold",""],["GenesisConfig",""],["HousingFundPalletId",""],["IdentityFee","Implementor of `WeightToFee` that maps one unit of weight to one unit of fee."],["InvestorVoteAmount",""],["ItemDeposit",""],["KeyLimit",""],["LaunchPeriod",""],["MaxAdditionalFields",""],["MaxApprovals",""],["MaxFundContribution",""],["MaxInvestorPerHouse",""],["MaxMembers",""],["MaxProposals",""],["MaxRegistrars",""],["MaxSubAccounts",""],["MaxTriesAseemblingInvestor",""],["MaxTriesBid",""],["MaximumReasonLength",""],["MaximumSchedulerWeight",""],["MaximumSharePerInvestor",""],["MetadataDepositBase",""],["MetadataDepositPerByte",""],["MinContribution",""],["MinimumDeposit",""],["MinimumSharePerInvestor",""],["NewAssetScanPeriod",""],["NoPreimagePostponement",""],["Origin","The runtime origin type representing the origin of a call."],["PalletId","A pallet identifier. These are per pallet and should be stored in a registry somewhere."],["PalletInfo","Provides an implementation of `PalletInfo` to provide information about the pallet setup in the runtime."],["Perbill","A fixed point representation of a number in the range [0, 1]."],["Permill","A fixed point representation of a number in the range [0, 1]."],["PreimageBaseDeposit",""],["PreimageByteDeposit",""],["PreimageMaxSize",""],["ProposalBond",""],["ProposalBondMinimum",""],["ProposalFee",""],["ReserveCollectionIdUpTo",""],["RocksDbWeight","By default, Substrate uses RocksDB, so this will be the weight used throughout the runtime."],["Runtime",""],["RuntimeApi",""],["RuntimeApiImpl","Implements all runtime apis for the client side."],["RuntimeBlockLength","We allow for 2 seconds of compute with a 6 second average block time."],["RuntimeBlockWeights",""],["SS58Prefix",""],["SimultaneousAssetBidder",""],["SpendPeriod",""],["StorageInfo","Metadata about storage from the runtime."],["StringLimit",""],["SubAccountDeposit",""],["TipCountdown",""],["TipFindersFee",""],["TipReportDepositBase",""],["TreasuryPalletId",""],["ValueLimit",""],["Version",""],["VotingPeriod",""],["Weight",""]],"trait":[["BuildStorage","Complex storage builder stuff."],["Contains","A trait for querying whether a type can be said to “contain” a value."],["KeyOwnerProofSystem","Something which can compute and check proofs of a historical key owner and return full identification data of that key owner."],["NftPermission",""],["Randomness","A trait that is able to provide randomness."],["StorageValue","A trait for working with macro-generated storage values under the substrate storage API."]],"type":[["Acc",""],["AccountId","Some way of identifying an account on the chain. We intentionally make it equivalent to the public key of our transaction signing scheme."],["Address","The address format for describing accounts."],["AllPallets","All pallets included in the runtime as a nested tuple of types."],["AllPalletsReversedWithSystemFirst","All pallets included in the runtime as a nested tuple of types in reversed order. With the system pallet first."],["AllPalletsWithSystem","All pallets included in the runtime as a nested tuple of types."],["AllPalletsWithSystemReversed","All pallets included in the runtime as a nested tuple of types in reversed order."],["AllPalletsWithoutSystem","All pallets included in the runtime as a nested tuple of types. Excludes the System pallet."],["AllPalletsWithoutSystemReversed","All pallets included in the runtime as a nested tuple of types in reversed order. Excludes the System pallet."],["AssetManagementModule",""],["Assets",""],["AssetsConfig",""],["Aura",""],["AuraConfig",""],["Balance","Balance of an account."],["Balances",""],["BalancesConfig",""],["BiddingModule",""],["Block","Block type as expected by this runtime."],["BlockNumber","An index to a block."],["CollectionId","NFT Collection ID"],["Council",""],["CouncilConfig",""],["Democracy",""],["DemocracyConfig",""],["Executive","Executive: handles dispatch to the various modules."],["FinalizerModule",""],["Grandpa",""],["GrandpaConfig",""],["Hash","A hash of some data used by the chain."],["Header","Block header type as expected by this runtime."],["HousingFundModule",""],["Identity",""],["Index","Index of a transaction in the chain."],["ItemId","NFT Item ID"],["NftModule",""],["NftModuleConfig",""],["OnboardingModule",""],["Preimage",""],["RandomnessCollectiveFlip",""],["RoleModule",""],["RoleModuleConfig",""],["Scheduler",""],["ShareDistributor",""],["Signature","Alias to 512-bit hash when used in the context of a transaction signature on the chain."],["SignedExtra","The SignedExtension to the basic transaction logic."],["SignedPayload","The payload being signed in transactions."],["Sudo",""],["SudoConfig",""],["System",""],["SystemConfig",""],["Timestamp",""],["TransactionPayment",""],["TransactionPaymentConfig",""],["Treasury",""],["TreasuryConfig",""],["UncheckedExtrinsic","Unchecked extrinsic type as expected by this runtime."],["Uniques",""],["Utility",""],["VotingModule",""]]};