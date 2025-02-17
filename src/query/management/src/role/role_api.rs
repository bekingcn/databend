// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use databend_common_exception::Result;
use databend_common_meta_app::principal::OwnershipInfo;
use databend_common_meta_app::principal::OwnershipObject;
use databend_common_meta_app::principal::RoleInfo;
use databend_common_meta_types::MatchSeq;
use databend_common_meta_types::SeqV;

#[async_trait::async_trait]
pub trait RoleApi: Sync + Send {
    async fn add_role(&self, role_info: RoleInfo) -> Result<u64>;

    #[allow(clippy::ptr_arg)]
    async fn get_role(&self, role: &String, seq: MatchSeq) -> Result<SeqV<RoleInfo>>;

    async fn get_roles(&self) -> Result<Vec<SeqV<RoleInfo>>>;

    /// General role update.
    ///
    /// It fetches the role that matches the specified seq number, update it in place, then write it back with the seq it sees.
    ///
    /// Seq number ensures there is no other write happens between get and set.
    #[allow(clippy::ptr_arg)]
    async fn update_role_with<F>(&self, role: &String, seq: MatchSeq, f: F) -> Result<Option<u64>>
    where F: FnOnce(&mut RoleInfo) + Send;

    /// Grant ownership would transfer ownership of a object from one role to another role
    ///
    ///
    /// Seq number ensures there is no other write happens between get and set.
    /// from: the role that currently owns the object, it could be empty on the first time
    /// to: the role that will own the object, to.0 is the owner role name, to.1 is the role details
    /// None RoleInfo means the role is built-in role, could only update grant object metadata
    #[allow(clippy::ptr_arg)]
    async fn grant_ownership(&self, object: &OwnershipObject, role: &str) -> Result<()>;

    /// Remember to call this method when you dropped a OwnerObject like table/database/stage/udf.
    async fn revoke_ownership(&self, object: &OwnershipObject) -> Result<()>;

    /// Get the ownership info by object. If it's not granted to any role, return PUBLIC
    async fn get_ownership(&self, object: &OwnershipObject) -> Result<Option<OwnershipInfo>>;

    async fn drop_role(&self, role: String, seq: MatchSeq) -> Result<()>;
}
