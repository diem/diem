/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import { LoadContext } from '@docusaurus/types';
import { MetadataRaw, MetadataOptions, Env } from './types';
declare type Args = {
    source: string;
    refDir: string;
    context: LoadContext;
    options: MetadataOptions;
    env: Env;
};
export default function processMetadata({ source, refDir, context, options, env, }: Args): Promise<MetadataRaw>;
export {};
//# sourceMappingURL=metadata.d.ts.map
