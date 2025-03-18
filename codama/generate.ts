import { createFromRoot } from "codama";
import { AnchorIdl, rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderJavaScriptVisitor } from "@codama/renderers";

import tokenTransferHookIdl from "../target/idl/token_transfer_hook.json";

const tokenTransferHookCodama = createFromRoot(
  rootNodeFromAnchor(tokenTransferHookIdl as AnchorIdl)
);

tokenTransferHookCodama.accept(
  renderJavaScriptVisitor("codama/token-transfer-hook/generated")
);

import rwaTokenizationIdl from "../target/idl/rwa_tokenization.json";

const rwaTokenizationCodama = createFromRoot(
  rootNodeFromAnchor(rwaTokenizationIdl as AnchorIdl)
);

rwaTokenizationCodama.accept(
  renderJavaScriptVisitor("codama/rwa-tokenization/generated")
);
