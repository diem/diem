/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
declare type FileLastUpdateData = {
    timestamp?: number;
    author?: string;
};
export default function getFileLastUpdate(filePath?: string): Promise<FileLastUpdateData | null>;
export {};
//# sourceMappingURL=lastUpdate.d.ts.map
