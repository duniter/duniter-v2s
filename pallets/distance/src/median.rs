// Copyright 2022 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

use frame_support::pallet_prelude::*;
use sp_std::cmp::Ordering;

/// Represents a median accumulator.
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo)]
pub struct MedianAcc<
    T: Clone + Decode + Encode + Ord + TypeInfo,
    const S: u32, /*Get<u32> + TypeInfo*/
> {
    samples: BoundedVec<(T, u32), ConstU32<S>>,
    median_index: Option<u32>,
    median_subindex: u32,
}

/*impl<T: 'static + Clone + Decode + Encode + Ord + TypeInfo, S: 'static + Get<u32>> TypeInfo
    for MedianAcc<T, S>
{
    type Identity = Self;

    fn type_info() -> scale_info::Type<scale_info::form::MetaForm> {}
}*/

/// Represents the result of a median calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MedianResult<T: Clone + Ord> {
    One(T),
    Two(T, T),
}

impl<T: Clone + Decode + Encode + Ord + TypeInfo, const S: u32 /*Get<u32> + TypeInfo*/>
    MedianAcc<T, S>
{
    pub fn new() -> Self {
        Self {
            samples: BoundedVec::default(),
            median_index: None,
            median_subindex: 0,
        }
    }

    pub fn push(&mut self, sample: T) {
        if let Some(median_index) = &mut self.median_index {
            match self
                .samples
                .binary_search_by_key(&sample, |(s, _n)| s.clone())
            {
                Ok(sample_index) => {
                    self.samples.get_mut(sample_index).expect("unreachable").1 += 1;
                    match (sample_index as u32).cmp(median_index) {
                        Ordering::Greater => {
                            if self.median_subindex
                                == self
                                    .samples
                                    .get(*median_index as usize)
                                    .expect("unreachable")
                                    .1
                                    * 2
                                    - 1
                            {
                                self.median_subindex = 0;
                                *median_index += 1;
                            } else {
                                self.median_subindex += 1;
                            }
                        }
                        Ordering::Equal => {
                            self.median_subindex += 1;
                        }
                        Ordering::Less => {
                            if self.median_subindex == 0 {
                                *median_index -= 1;
                                self.median_subindex = self
                                    .samples
                                    .get(*median_index as usize)
                                    .expect("unreachable")
                                    .1
                                    * 2
                                    - 1;
                            } else {
                                self.median_subindex -= 1;
                            }
                        }
                    }
                }
                Err(sample_index) => {
                    self.samples.try_insert(sample_index, (sample, 1)).ok();
                    if *median_index as usize >= sample_index {
                        if self.median_subindex == 0 {
                            self.median_subindex = self
                                .samples
                                .get(*median_index as usize)
                                .expect("unreachable")
                                .1
                                * 2
                                - 1;
                        } else {
                            self.median_subindex -= 1;
                            *median_index += 1;
                        }
                    } else if self.median_subindex
                        == self
                            .samples
                            .get(*median_index as usize)
                            .expect("unreachable")
                            .1
                            * 2
                            - 1
                    {
                        self.median_subindex = 0;
                        *median_index += 1;
                    } else {
                        self.median_subindex += 1;
                    }
                }
            }
        } else {
            self.samples.try_push((sample, 1)).ok();
            self.median_index = Some(0);
        }
    }

    pub fn get_median(&self) -> Option<MedianResult<T>> {
        self.median_index.map(|median_index| {
            let (median_sample, median_n) = self
                .samples
                .get(median_index as usize)
                .expect("unreachable");
            if self.median_subindex == median_n * 2 - 1 {
                MedianResult::Two(
                    median_sample.clone(),
                    self.samples
                        .get(median_index as usize + 1)
                        .expect("unreachable")
                        .0
                        .clone(),
                )
            } else {
                MedianResult::One(median_sample.clone())
            }
        })
    }
}
